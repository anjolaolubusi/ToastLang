#ifndef CODEGEN_H
#define CODEGEN_H
#include "astNode.h"
#include "tokens.h"
#include "llvm/IR/Value.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Constant.h"
#include "llvm/IR/Verifier.h"
#include "llvm/Transforms/Scalar.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/Transforms/InstCombine/InstCombine.h"
#include "llvm/Transforms/Scalar/GVN.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/IRPrinter/IRPrintingPasses.h"
#include "llvm/IR/PassManager.h"

struct CodeGenerator: CodeVisitor{
    std::unique_ptr<llvm::LLVMContext> TheContext;
    std::unique_ptr<llvm::IRBuilder<>> Builder;
    std::unique_ptr<llvm::Module> TheModule;
    std::map<std::string, llvm::Value*> NamedValues;

    CodeGenerator(){
        TheContext = std::make_unique<llvm::LLVMContext>();
        TheModule = std::make_unique<llvm::Module>("ToastLang", *TheContext);
        Builder = std::make_unique<llvm::IRBuilder<>>(*TheContext);
        llvm::FunctionCallee CalleeF = TheModule->getOrInsertFunction("printTest", llvm::FunctionType::get(llvm::Type::getFloatTy(*TheContext), true /* this is var arg func type*/));
        int test = CalleeF.getFunctionType()->getNumParams();
        int temp = 0;
        //MPM.addPass(lldvm::PrintModulePass());
    }

    void Optimize(){
        llvm::PassBuilder PB;
        llvm::LoopAnalysisManager LAM;
        llvm::FunctionAnalysisManager FAM;
        llvm::CGSCCAnalysisManager CGAM;
        llvm::ModuleAnalysisManager MAM;
        llvm::ModulePassManager MPM;
        llvm::FunctionPassManager FPM;

        //MPM.addPass(llvm::createModuleToFunctionPassAdaptor(std::move(FPM)));

        PB.registerModuleAnalyses(MAM);
        PB.registerCGSCCAnalyses(CGAM);
        PB.registerFunctionAnalyses(FAM);
        PB.registerLoopAnalyses(LAM);
        PB.crossRegisterProxies(LAM, FAM, CGAM, MAM);
        MPM = PB.buildPerModuleDefaultPipeline(llvm::OptimizationLevel::O2);
        FPM = PB.buildFunctionSimplificationPipeline(llvm::OptimizationLevel::O2, llvm::ThinOrFullLTOPhase::FullLTOPreLink);
        MPM.run(*TheModule, MAM);
    }

    //Logs Compile Error
    llvm::Value* LogErrorV(const char *str){
        fprintf(stderr, "Compile Error: %s\n", str);
        return nullptr;
    }

    //Compiles number as float
    llvm::Value* visit(NumberExpr& numExpr) override {
        return llvm::ConstantFP::get(*TheContext, llvm::APFloat(std::stof(numExpr.number)));
    }

    //Grabs variable from memory
    llvm::Value* visit(VariableExpr& varExpr) override {
        llvm::Value* V = NamedValues[varExpr.varName];
        if(!V){
            LogErrorV("Unknown variable name");
        }
        return V;
    }

    //Checks the binary op and write the correct instructions
    llvm::Value* visit(BinaryExpr& binExpr) override{
        llvm::Value* L = binExpr.lhs->compile(*this);
        llvm::Value* R = binExpr.rhs->compile(*this);
        if(!L || !R){
            return nullptr;
        }

        switch (binExpr.op)
        {
        case tok_plus:
            return Builder->CreateFAdd(L, R, "addtmp");
        case tok_minus:
            return Builder->CreateFSub(L, R, "subtmp");
        case tok_multiply:
            return Builder->CreateFMul(L, R, "multmp");
        case tok_lessThan:
            return Builder->CreateUIToFP(L, llvm::Type::getFloatTy(*TheContext), "booltmp");
        case tok_customBinOP:
            return nullptr;
        default:
            return LogErrorV("Invalid binary operator");
        }
    }

    //Grab function from memory and calls it
    llvm::Value* visit(CallExpr& callExpr) override{
        llvm::Function* CalleeF = TheModule->getFunction(callExpr.funcName);
        if(!CalleeF){
            return LogErrorV("Unknown function referenced");
        }

        if(CalleeF->arg_size() != callExpr.parameters.size()){
            return LogErrorV("Incorrect # arguments passed");
        }

        std::vector<llvm::Value*> ArgsV;
        for(unsigned int i = 0, e = callExpr.parameters.size(); i != e; i++){
            ArgsV.push_back(callExpr.parameters[i]->compile(*this));
            if(!ArgsV.back()){
                return nullptr;
            }
        }
        return Builder->CreateCall(CalleeF, ArgsV, "calltmp");
    }

    //Creates brances between blocks 
    llvm::Value* visit(IfExpr& ifExpr) override{
        llvm::Value* CondV = ifExpr.condExpr->compile(*this);
        if(!CondV){
            return LogErrorV("Could not compile condition");
        }
        CondV = Builder->CreateFCmpONE(CondV, llvm::ConstantFP::get(*TheContext, llvm::APFloat(0.0f)), "ifcond");
        llvm::Function* TheFunction = Builder->GetInsertBlock()->getParent();
        llvm::BasicBlock* ThenBB = llvm::BasicBlock::Create(*TheContext, "then", TheFunction);
        llvm::BasicBlock* ElseBB = llvm::BasicBlock::Create(*TheContext, "else", TheFunction);
        llvm::BasicBlock* MergeBB = llvm::BasicBlock::Create(*TheContext, "ifcont", TheFunction);
        Builder->CreateCondBr(CondV, ThenBB, ElseBB);
        Builder->SetInsertPoint(ThenBB);
        llvm::Value* ThenV = ifExpr.thenExpr->compile(*this);
        if(!ThenV){
            return LogErrorV("Can not compile then block");
        }
        Builder->CreateBr(MergeBB);
        ThenBB = Builder->GetInsertBlock();

        TheFunction->insert(TheFunction->end(), ElseBB);
        Builder->SetInsertPoint(ElseBB);
        llvm::Value* ElseV;
        if(ifExpr.elseExpr == nullptr){
            ElseV = NumberExpr("0").compile(*this);
        }else{
            ElseV = ifExpr.elseExpr->compile(*this);
        }
        if(!ElseV){
            return LogErrorV("Can not compile else block");
        }
        Builder->CreateBr(MergeBB);
        ElseBB = Builder->GetInsertBlock();
        

        TheFunction->insert(TheFunction->end(), MergeBB);
        Builder->SetInsertPoint(MergeBB);
        llvm::PHINode *PN = Builder->CreatePHI(llvm::Type::getFloatTy(*TheContext), 2, "iftmp");
        PN->addIncoming(ThenV, ThenBB);
        PN->addIncoming(ElseV, ElseBB);
        return PN;
    }

    llvm::Value* visit(ForExpr& forExpr) override{
        llvm::Value* StartV = forExpr.start->compile(*this);
        if(!StartV){
            LogErrorV("Can not compile start value in for loop");
        }
        llvm::Function* TheFunction  = Builder->GetInsertBlock()->getParent();
        llvm::BasicBlock* PreheaderBB = Builder->GetInsertBlock();
        llvm::BasicBlock* LoopBB = llvm::BasicBlock::Create(*TheContext, "loop", TheFunction);
        Builder->CreateBr(LoopBB);
        Builder->SetInsertPoint(LoopBB);

        llvm::PHINode* Variable = Builder->CreatePHI(llvm::Type::getFloatTy(*TheContext), 2, forExpr.var.c_str());
        Variable ->addIncoming(StartV, PreheaderBB);
        llvm::Value* OldV = NamedValues[forExpr.var];
        NamedValues[forExpr.var] = Variable;

        if(!forExpr.body->compile(*this)){
            return nullptr;
        }

        llvm::Value* StepV = nullptr;
        if(forExpr.stepFunc){
            StepV = forExpr.stepFunc->compile(*this);
            if(!StepV){
                return LogErrorV("Failed to compile for loop");
            }
        }else{
            StepV = llvm::ConstantFP::get(*TheContext, llvm::APFloat(1.0));
        }
        llvm::Value* NextVar = Builder->CreateFAdd(Variable, StepV, "nextvar");

        llvm::Value* EndCond = forExpr.cond->compile(*this);
        if(!EndCond){
            return LogErrorV("Can not compile end condition in for loop");
        }
        EndCond = Builder->CreateFCmpONE(EndCond, llvm::ConstantFP::get(*TheContext, llvm::APFloat(0.0)), "loopcond");

        llvm::BasicBlock* LoopEndBB = Builder->GetInsertBlock();
        llvm::BasicBlock* AfterBB = llvm::BasicBlock::Create(*TheContext, "afterloop", TheFunction);
        Builder->CreateCondBr(EndCond, LoopBB, AfterBB);
        Builder->SetInsertPoint(AfterBB);
        Variable->addIncoming(NextVar, LoopEndBB);

        if(OldV){
            NamedValues[forExpr.var] = OldV;
        }else{
            NamedValues.erase(forExpr.var);
        }

        return llvm::Constant::getNullValue(llvm::Type::getFloatTy(*TheContext));
    }

    //Creates function with empty body in memory
    llvm::Function* visit(ProtoAST& protoAST) override{
        std::vector<llvm::Type*> Doubles(protoAST.args.size(), llvm::Type::getFloatTy(*TheContext));

        llvm::FunctionType* FT = llvm::FunctionType::get(llvm::Type::getFloatTy(*TheContext), Doubles, false);
        llvm::Function* F = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, protoAST.name, TheModule.get());
        
        unsigned Idx = 0;
        for(auto &Arg: F->args()){
            Arg.setName(protoAST.args[Idx]);
        }

        return F;

    }

    //Compiles function
    llvm::Function* visit(FuncAST& funcAST) override{
        llvm::Function* TheFunction = TheModule->getFunction(funcAST.Proto->name);
        if(!TheFunction){
            TheFunction = funcAST.Proto->compile(*this);
        }

        if(!TheFunction){
            return nullptr;
        }

        if(!TheFunction->empty()){
            return (llvm::Function*) LogErrorV("Fuunction cannot be redefined");
        }

        llvm::BasicBlock *BB = llvm::BasicBlock::Create(*TheContext, "entry", TheFunction);
        Builder->SetInsertPoint(BB);

        NamedValues.clear();
        for(auto &Arg: TheFunction->args()){
            NamedValues[Arg.getName().data()] = &Arg;
        }

        if(llvm::Value* RetVal = funcAST.Body->compile(*this)){
            Builder->CreateRet(RetVal);
            llvm::verifyFunction(*TheFunction);
            Optimize();
            //TheFunction->viewCFGOnly();
            return TheFunction;
        }
        TheFunction->eraseFromParent();
        return nullptr;
    }

    //Compiles command as a function with no parameters
    llvm::Function* visit(ExprNode& exprNode) override{
        std::vector<llvm::Type*> Doubles(0, llvm::Type::getFloatTy(*TheContext));
        std::string anonFuncName = "anonexpr_";
        llvm::FunctionType* FT = llvm::FunctionType::get(llvm::Type::getFloatTy(*TheContext), Doubles, false);
        llvm::Function* F = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, anonFuncName.c_str(), TheModule.get());
        
        if(!F){
            return nullptr;
        }

        if(!F->empty()){
            return (llvm::Function*) LogErrorV("Fuunction cannot be redefined");
        }

        llvm::BasicBlock *BB = llvm::BasicBlock::Create(*TheContext, "entry", F);
        Builder->SetInsertPoint(BB);

        if(llvm::Value* RetVal = exprNode.expr->compile(*this)){
            Builder->CreateRet(RetVal);
            llvm::raw_ostream &output = llvm::outs();
            llvm::verifyFunction(*F, &output);
            //F->viewCFGOnly();
            //FPM.run(*F, FAM);
            //Optimize();
            //MPM.run(*TheModule, this->MAM);
            //F->viewCFGOnly();
            //RetVal->print(llvm::outs());
            // fprintf(stdout, "Evaluated to: ");
            // fprintf(stdout, "\n");
            // RetVal->print(llvm::outs());
            // fprintf(stdout, "\n");
            return F;
        }
        F->eraseFromParent();
        return nullptr;
    }

};

#endif
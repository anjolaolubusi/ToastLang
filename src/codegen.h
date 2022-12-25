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

struct CodeGenerator: CodeVisitor{
    std::unique_ptr<llvm::LLVMContext> TheContext;
    std::unique_ptr<llvm::IRBuilder<>> Builder;
    std::unique_ptr<llvm::Module> TheModule;
    std::map<std::string, llvm::Value*> NamedValues;
    llvm::PassBuilder PB;
    llvm::LoopAnalysisManager LAM;
    llvm::FunctionAnalysisManager FAM;
    llvm::CGSCCAnalysisManager CGAM;
    llvm::ModuleAnalysisManager MAM;
    llvm::ModulePassManager MPM;

    //std::unique_ptr<llvm::legacy::FunctionPassManager> TheFPM;


    CodeGenerator(){
        TheContext = std::make_unique<llvm::LLVMContext>();
        TheModule = std::make_unique<llvm::Module>("ToastLang", *TheContext);
        Builder = std::make_unique<llvm::IRBuilder<>>(*TheContext);
        PB.registerModuleAnalyses(MAM);
        PB.registerCGSCCAnalyses(CGAM);
        PB.registerFunctionAnalyses(FAM);
        PB.registerLoopAnalyses(LAM);
        PB.crossRegisterProxies(LAM, FAM, CGAM, MAM);
        MPM = PB.buildPerModuleDefaultPipeline(llvm::OptimizationLevel::O2);
        // TheFPM = std::make_unique<llvm::legacy::FunctionPassManager>(TheModule.get());
        // TheFPM->add(llvm::createInstructionCombiningPass());
        // TheFPM->add(llvm::createReassociatePass());
        // TheFPM->add(llvm::createGVNPass());
        // TheFPM->add(llvm::createCFGSimplificationPass());
        // TheFPM->doInitialization();
    }

    llvm::Value* LogErrorV(const char *str){
        fprintf(stderr, "Compile Error: %s\n", str);
        return nullptr;
    }

    llvm::Value* visit(NumberExpr& numExpr) override {
        return llvm::ConstantFP::get(*TheContext, llvm::APFloat(std::stof(numExpr.number)));
    }

    llvm::Value* visit(VariableExpr& varExpr) override {
        llvm::Value* V = NamedValues[varExpr.varName];
        if(!V){
            LogErrorV("Unknown variable name");
        }
        return V;
    }

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
            return Builder->CreateUIToFP(L, llvm::Type::getDoubleTy(*TheContext), "booltmp");
        case tok_customBinOP:
            return nullptr;
        default:
            return LogErrorV("Invalid binary operator");
        }
    }

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

    llvm::Function* visit(ProtoAST& protoAST) override{
        std::vector<llvm::Type*> Doubles(protoAST.args.size(), llvm::Type::getDoubleTy(*TheContext));

        llvm::FunctionType* FT = llvm::FunctionType::get(llvm::Type::getDoubleTy(*TheContext), Doubles, false);
        llvm::Function* F = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, protoAST.name, TheModule.get());
        
        unsigned Idx = 0;
        for(auto &Arg: F->args()){
            Arg.setName(protoAST.args[Idx]);
        }

        return F;

    }

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
            MPM.run(*TheModule, this->MAM);
            // TheFPM->run(*TheFunction);
            return TheFunction;
        }
        TheFunction->eraseFromParent();
        return nullptr;
    }

    llvm::Function* visit(ExprNode& exprNode) override{
        std::vector<llvm::Type*> Doubles(0, llvm::Type::getDoubleTy(*TheContext));
        std::string anonFuncName = "anonexpr_";
        llvm::FunctionType* FT = llvm::FunctionType::get(llvm::Type::getDoubleTy(*TheContext), Doubles, false);
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
            if(!llvm::verifyFunction(*F)){
                fprintf(stdout, "Error when compiling function: \n");
                F->print(llvm::errs());
            };
            // TheFPM->run(*F);
            return F;
        }
        F->eraseFromParent();
        return nullptr;
    }


};

#endif
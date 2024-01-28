# ToastLang
Functional Programming Language with an easier synatax to learn.


## Grammar Description
Here is the grammar description. This is likely to change.
| Non-terminals    | Description                                                  |
|------------------|--------------------------------------------------------------|
| program          | [[statment \| expression] ]*                                 |
| statement        | [declaration \| definition]                                  |
| declaration      | Extern prototype                                             |
| definition       | Def prototype Colon expression End                                 |
| prototype        | Ident OpeningParenthesis [Ident Comma ?]* ClosingParenthesis |
| expression       | [primary_expr (operator primary_expr)*]                            |
| operator         | Plus \| Minus \| Multiply \| Divide | Modulus                             |
| primary_expr     | [Ident \| Number \| call_expr \| parenthesis_expr]           |
| call_expr        | OpeningParenthesis [Ident Comma ?]* ClosingParenthesis       |
| parenthesis_expr | OpeningParenthesis expression ClosingParenthesis             |
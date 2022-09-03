# ToastLang

## Grammar Description
Here is the grammar description. This is likely to change.
| Non-terminals    | Description                                                  |
|------------------|--------------------------------------------------------------|
| program          | [[statment \| expression] ]*                                 |
| statement        | [declaration \| definition]                                  |
| declaration      | Extern prototype                                             |
| definition       | Def prototype expression End                                 |
| prototype        | Ident OpeningParenthesis [Ident Comma ?]* ClosingParenthesis |
| expression       | [primary_expr (Op primary_expr)*]                            |
| primary_expr     | [Ident \| Number \| call_expr \| parenthesis_expr]           |
| call_expr        | OpeningParenthesis [Ident Comma ?]* ClosingParenthesis       |
| parenthesis_expr | OpeningParenthesis expression ClosingParenthesis             |
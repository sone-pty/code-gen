@keyword_literal = 101;
@symbol_literal = 102;

token
    : '@' @ident '=' @integer ';'
    ;

item
    : @ident
    | @keyword_literal
    | @symbol_literal
    | '@' @ident
    | @ident '<' '>'
    | @ident '<' node_param_list '>'
    | @ident '<' node_param_list ',' '>'
    | @ident '<' '@' node_modifier_param_list '>'
    | @ident '<' '@' node_modifier_param_list ',' '>' 
    ;

node_param_list
    : @ident
    | node_param_list ',' @ident
    ;

node_modifier_param
    : @ident
    | '!' @ident
    ;

node_modifier_param_list
    : node_modifier_param
    | node_modifier_param_list ',' node_modifier_param
    ;

followed_item
    : item
    | "_" @keyword_literal
    | "_" @symbol_literal
    | "_" '@' @ident
    ;

followed_item_list
    : followed_item
    | followed_item_list followed_item
    ;

not_followed_item
    : @keyword_literal
    | @symbol_literal
    | '@' @ident
    | "_" @keyword_literal
    | "_" @symbol_literal
    | "_" '@' @ident
    ;

not_followed_item_list
    : not_followed_item
    | not_followed_item_list not_followed_item
    ;

statement
    : item
    | item '!' not_followed_item_list
    | item followed_item_list
    | item followed_item_list '!' not_followed_item_list
    ;

statement_with_cond
    : statement
    | statement '^' @ident
    | statement '^' '!' @ident
    ;

statement_list
    : statement_with_cond
    | statement_list '|' statement_with_cond
    ;

production
    : @ident ':' statement_list ';'
    | '#' @ident ':' statement_list ';'
    ;

import
    : "mod" @ident ';'
    ;


script_item
    : token
    | production
    | import
    ;

script
    : script_item
    | script script_item
    ;
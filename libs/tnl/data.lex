

#value
    : "null"
    | "true"
    | "false"
    | @integer
    | '-' @integer
    | @float
    | '-' @float
    | @string
    | @code_block
    | object
    | array
    | @ident
    ;

array
    : '[' ']'
    | '[' value_list ']'
    ;

value_list 
    : value
    | value_list value
    | value ','
    | value_list value ','
    ;

object_item
    : @ident ':' value
    | value
    | @ident ':' value ','
    | value ','
    ;

#object_item_list
    : object_item
    | object_item_list object_item
    ;

object
    : '{' '}'
    | '@' @ident
    | '@' @ident '{' '}'
    | '{' object_item_list '}'
    | '@' @ident '{' object_item_list '}'
    | '@' @ident ':' @ident
    | '@' @ident ':' @ident '{' '}'
    | '@' @ident ':' @ident '{' object_item_list '}'
    ;

@bin_integer = 201;
@oct_integer = 202;
@hex_integer = 203;
@integer_with_exponent = 204;

bool_literal
    : "true"
    | "TRUE"
    | "FALSE"
    | "false"
    ;

integer_literal
    : @bin_integer
    | @bin_integer _ @ident
    | @oct_integer
    | @oct_integer _ @ident
    | @integer
    | @integer ! _ '.'
    | @integer ! _ '..'
    | @integer _ @ident
    | @hex_integer
    | @hex_integer _ @ident
    ;

float_literal
    : @integer_with_exponent
    | @integer_with_exponent _ @ident
    | @integer _ '.' _ @integer
    | @integer _ '.' _ @integer _ @ident
    | @integer _ '.' _ @integer_with_exponent
    | @integer _ '.' _ @integer_with_exponent _ @ident
    ;

null_value
    : "null"
    | "NULL"
    ;

literal_vals
    : bool_literal<>
    | integer_literal<>
    | float_literal<>
    | @string
    ;

array_vals
    : '{' '}'
    | '{' array_elements<> '}'
    | '{' array_elements<> ',' '}'
    ;

array_elements
    : values<>
    | array_elements<> ',' values<>
    ;

#values
    : literal_vals<>
    | array_vals<>
    | @ident
    | null_value<>
    ;

float_type
    : "float"
    | "f32"
    | "FLOAT"
    ;

double_type
    : "double"
    | "f64"
    | "DOUBLE"
    ;

bool_type
    : "bool"
    | "BOOL"
    ;

short_type
    : "short"
    | "SHORT"
    ;

ushort_type
    : "ushort"
    | "USHORT"
    ;

int_type
    : "int"
    | "INT"
    ;

uint_type
    : "uint"
    | "UINT"
    ;

byte_type
    : "byte"
    | "BYTE"
    ;

sbyte_type
    : "sbyte"
    | "SBYTE"
    ;

custom_type
    : @ident
    | custom_type<> '.' @ident
    ;

enum_type
    : "enum" @ident '.' @ident
    | "ENUM" @ident '.' @ident
    ;

tuple_type
    : "Tuple" '<' tuple_type_elements<> '>'
    | "Tuple" '<' tuple_type_elements<> ',' '>'
    ;

value_tuple_type
    : "ValueTuple" '<' tuple_type_elements<> '>'
    | "ValueTuple" '<' tuple_type_elements<> ',' '>'
    ;

tuple_type_elements
    : value_type<>
    | tuple_type_elements<> ',' value_type<>
    ;

short_list_type
    : "ShortList"
    | "shortlist"
    ;

lstring_type
    : "LString"
    | "lstring"
    | "Lstring"
    ;

decimal_type
    : "decimal"
    | "Decimal"
    ;

string_type
    : "string"
    | "String"
    ;

list_type
    : "List" '<' value_type<> '>'
    ;

array_type
    : value_type<> '[' ']'
    | value_type<> '[' integer_literal<> ']'
    ;

#value_type
    : decimal_type<>
    | float_type<>
    | double_type<>
    | int_type<>
    | uint_type<>
    | short_type<>
    | ushort_type<>
    | lstring_type<>
    | array_type<>
    | list_type<>
    | short_list_type<>
    | string_type<>
    | value_tuple_type<>
    | bool_type<>
    | custom_type<>
    | enum_type<>
    | tuple_type<>
    | byte_type<>
    | sbyte_type<>
    ;

#assign
    : value_type<> '=' values<>
    | value_type<> ':' values<>
    ;
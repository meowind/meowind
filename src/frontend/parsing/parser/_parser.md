# currently supported features:
## expressions
### binary
structure: `<expr> <op> <expr>`

#### arithmetic operators
| Name          | Token         |
| ------------- | ------------- |
| `add` | `+` |
| `subtract` | `-` |
| `multiply` | `*` |
| `divide` | `/` |
| `modulo` | `%` |
| `power` | `**` |

#### relational operators
| Name          | Token         |
| ------------- | ------------- |
| `equal` | `==` |
| `not equal` | `!=` |
| `greater` | `>` |
| `less` | `<` |
| `greater or equal` | `>=` |
| `less or equal` | `<=` |

#### logical operators
| Name          | Token         |
| ------------- | ------------- |
| `and` | `&&` |
| `or` | `\|\|` |

### unary
structure: `<op> <expr>`

#### operators
| Name          | Token         |
| ------------- | ------------- |
| `arithmetic negation` | `-` |
| `logical negation` | `!` |

### literal
#### supported literals
| Literal          | Example         |
| ------------- | ------------- |
| `integer` | `10` |
| `float` | `50.3` |
| `string` | `"hello world"` |
| `boolean` | `true` |

### identifier
example: `my_var`

### call
structure: `<expr>([<expr>, ...])`

### resolution
example: `namespace_a::namespace_b::some_item.some_member`
#### resolution kinds
| Name          | Token         | Example |
| ------------- | ------------- | ------------- |
| `namespace` | `::` | `namespace_a::namespace_b` |
| `member` | `.` | `some_item.some_member` |

### assignments
structure: `<expr> <ass> <expr>`

#### assignments
| Name          | Token         |
| ------------- | ------------- |
| `straight` | `=` |
| `plus equals` | `+=` |
| `minus equals` | `-=` |
| `multiply equals` | `*=` |
| `divide equals` | `/=` |
| `modulo equals` | `%=` |
| `power equals` | `**=` |

## items
### constants
structure: `[pub] const <name>: <type> = <expr>;`\
examples:
```
const a: int32 = 50;
pub const b: string = "hello world!";
```
### static variables
structure: `[pub] static [mut] <name>[: <type>] = <expr>;`\
examples:
```
static a = 60;
pub static b = 50 + 50;
pub static mut c = "hello " + "world!";
pub static mut d: bool = true;
```
### functions
structure: `[pub] func <name>([<name>[: <type>] [= <expr>], ...]) [-> <type> | <name>: <type>] <body>`\
examples:
```
func a() { }
pub func b() { }
pub func c(x: int32) { }
pub func d(x: int32) -> string { }
pub func e(x: int32) -> output: string { }
```
## bodies
### body
structure: `<stmt> => <expr|stmt|body>;`
### multiline body
structure: `<stmt> { [<expr|stmt|body>; ...] }`
## statements
### variable declaration
structure: `let [mut] <name>[: <type>] [= <expr>];`\
examples:
```
let a: int32;
let b = 50;
let c: int32 = 50;
let mut d: bool = true;
```
### function declaration
same as function declaration in items, but without `pub` keyword, [read here](#functions)

### if statement
structure: `if <expr> <body> [else <if statement> | else <body>]`
example:
```
if a == 0 { }
else if a == 1 => ;
else if a == 2 { }
else { }
```
### while loop
structure: `while <expr> <body> [else <while loop> | else <body>]`
example:
```
while a == 0 { }
else while a == 1 => ;
else while a == 2 { }
else { }
```

### return
structure: `return <expr>;`
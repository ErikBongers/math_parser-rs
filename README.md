12/2023: CURRENTLY CONVERTING FROM CPP TO RUST.
The online project will still be using the cpp code base.

[![experimental](http://badges.github.io/stability-badges/dist/experimental.svg)](http://github.com/badges/stability-badges)
# MathParser
## Introduction
Mathparser is a mathematical expression language based on C-style syntax.
It was created out of a desire for a simple mathematical script language outside of spreadsheets, online c-compilers, mathlab and other math tools.
I think it may feel more familiar to programmers than those other tools.

## Language reference

### Basic syntax
```
//this is a comment line
a=3+4; //this is a typical statement.
b=2a+3; //implicit multiplication is allowed: same as (2*a)+3;
a+=2; b=a*10; //a statement always ends with a semi-colon, not a new line, so these are two statements.
randomNumbers = 1, 234, 567; // a list of numbers (array)
```

### Numeric notation
A dot is the decimal separator, but in the section [Formatted values](#formatted-values) there's an alternative.
```
123.456; 
0xFF; //hexadecimal notation for decimal 255
0b101; //binary notation for decimal 5
123e4; //scientific notation for decimal 1230000
```

The type of notation for a variable is remembered for as long as possible. That is, MathParser will try to output the value in the initial format.

### Operators
In addition to the usual `+ - * /` operators, there are:
```
a+=7; //same as a=a+7, also works with - * /
a=3^2; //exponent operator
a=|-3|; //absolute value operator
a++; //increment operator: same as a=a+1 or a+=1;
a--; //decrement operator.
a%3; //remainder operator: -15%12 = -3;
a%%3; //modulus operator: -15%%12 = 9; //very usefull for dates and hours: starting from midnight, -15 hours = 9:00 , not -3:00 !
3!; //factorials.
```
### Output control
```
a=2*7; //will output the variable and it's value: a=14
7*3; //will output the value: 21
```

##### Echo
An exclamation mark `!` is used to echo the full expression to the output.
```
!b=60/3; //Will echo the code together with the result, but not the comment --> b=20 b=60/3
!//this comment line will appear on the output
!q=100/10; !//some comment  --> output: q=10 q=100/10 //some comment
!!q=100/10; //some comment  --> echo the full line (same effect as the line above)
!/// --> echo all the input for all the lines until...
///!  --> end of echo
```
##### Mute
A hashtag `#` is used to mute the result.
```
#c=sin(30deg); //suppress output, but still execute the code.
!#d=a+b; //execute the statement, echo the code, but mute the result
/# //suppress output for the next 4 lines, with intermediate results
a+=1;
a+=3;
a+=PI;
a+=7;
#/
a; //finally, the result will be output
```

### Units
Units can be appended either directly to numbers or seperated by a dot for identifiers.
Values get automatically converted to the last unit.
```
12kg;
twelve.kg;
Distance=10km+1mi; //Distance will be expressed in the first unit: km.

//Conversion
Distance=100; //Standard unit (m) implied.
Imperial=Distance.mi; //Conversion from meter to miles.
Result=(Imperial+5m).km; //Conversion of an expression.
Hot=1000K.C.F; //Conversion from K to C to F. Note that the intermediate conversion to C is pointless.

//Only output or conversion?
Hot=1000K;
Hot.C; //Output the value of Hot in Celcius. The variable Hot remains in K!
Hot=Hot.C; //Convert Hot to Celcius.
Hot.=C; //Does the same as the above line.
```
#### Implemented units:
* Angle: `rad, deg`
* Length: `km, m, cm, mm, um (micron), in, ft, mi, thou, yd`
* Temperature: `C, F, K`
* Mass (weight): `kg, g, mg, t, lb (lbs), oz, N` (note that for convenience no distinction is made between weight and mass)
* Volume: `L, ml, gal, pt`

### Formats
Like units, an output format can be specified with dot notation.
```
123.dec; // 123 (default) 
123.hex; // 0x7B
123.oct; // 0o173
123.bin; // 0b1111011
123.exp; // 123e0
```
### Built in functions
* Trigonometry: `sin, cos, tan, asin, acos, atan`
* Other: `round, floor, ceil, trunc, abs, factors`
* ```max(randomNumbers); // lists (arrays) can also be used as arguments```
* `|x|` is the same as `abs(x)`
* Dates: `now(), date(year, month, day)`
* Lists: `sort(), reverse(), max(), min(), avg()`

### Custom functions
Statements can be grouped in functions as well.
```
x = 100;
function hundred(a)
  {
  cent = x;//error: variables from outside the function are not visible within the function body (scope).
  cent = 100; //ok
  a*cent; //the last statement defines the return value of the function.
  }
 cent++;//error: cent was a local variable of hundred(). It's out of scope here.
 x=hundred(1+2); // = 300
```
### Constants
Currently only PI. (also in lower case)
[TODO]: G, C, e...

### Formatted values
Formatted values are values between single quotes that are guessed what they may mean and allow for different locales.
```
a='01/jan/2022'; //interpreted as a date.
#define decimal_comma //use a comma as a decimal point and a dot as a thousands separator
european_value = '123.456,78';
#define decimal_dot 
american_value = '123,456.78';
```
Since these values are guessed, there is no guarantee that they are interpreted the way you intended too, so use with care.
However, if a value is clearly ambiguous, that is, if it can be interpreted in multiple ways, an error will be reported.

### Dates
#### Concepts
Math Parser follows to some extend the chrono library concepts.
* A **date** is a point in time. It has no length or duration. It has the members: day, month, year.
* A **duration** is a length of time. You can add and multiply them. It has the members: days, months, years. Note the plural form.
#### Calculations
```
date - date ==> duration
date + duration ==> date
duration * x ==> duration // or any other arithmetic
```
#### Dates (points in time)
Date values can be written as formatted values.
Math Parser will try to parse any date format, as long as it's not ambiguous.
```
a='01/jan/2022'; 
a='2022/22/11'; 
a='2022/11/22'; 

meh='11/11/11'; //this works.
duh='1/1/2022'; //still works...
really='11/2022/11'; //yep, just fine...

a='2022/12/11'; //ambiguous
a='2/1/2022'; //ambiguous
```
Or, dates can be created by assigning a comma-separated list of values:
```
#define dmy; // or ymd or mdy
day = 23;
month = 12;
a_day = date(day, month/2, now().year); // june 23, of this year
```
The parts of a date can be referenced, but not assigned to:
```
thisMonth = now().month;
myDate.year=2022; //error: can't assign to a date
```
Thus, allowing for calculated values.
Note that you must `#define` a strict date format, since changes in calculations could lead to the values suddenly being interpreted in a different order than what you intended.
[TODO]: allow access to date parts and date calculations, enforce a strict date format. Implement time.

#### Durations (lenght of time)
A typical duration would be my age:
```
birthday=1968, 7, 30;
age=now()-birthday;
```

### Settings
Some settings define how the parser behaves.
These can be changed with ```#define``` and ```#undef```
```
#define dmy // date formats can be dmy, mdy or ymd
        trig //activate trigonometry functions,
        date_units //day, month, year
        short_date_units // d, M, y
        arythm //abs(), round(),...
        date  //date(), now()
        all // all functions
        electric //numeric notations for resistors and capacitors
        strict //trig functions will require params to have the units deg or rad where applicable.
        decimal_dot // set decimal charater and thousands separaterd in a formatted string: american_value = '123.456,67';
        dec_dot  //short form
        dot  //shorter
        decimal_comma //same as above, other locale: european_value = '123.456,78';
        dec_comma
        comma
```

### Scope
A scope is defined by a code block: ``` { ... }```. Within a scope, the external 'world' is unknown and vice-versa. A code block is thus a completely stand-alone program.
```
a=1;
{
b=a; //error: 'a' is unknown as it is external to the block.
c =3;
}
d=c; //error: 'c' is unknown outside of the block.
```

Code blocks allow to have a little program that 'does it's own thing' within a larger file.
A typical use is to temporarily change some settings:
```
#define decimal_dot
theDot = '1,234.56';
{
#define decimal_comma
theDot = '1.234,56';
}
//a dot is still the decimal char in the outer scope:
theDot = '1,234.56';
```

## Technical
The main parser project is **MathParserDll** and is written in C++. It is a homebrew recursive descent parser with 2 look-aheads. Advantages of this parser type are that it's intuitive to read and mimicks the grammar definition (EBNF).

### Projects
* **MathParserDll** main parser project.
* **MathParserWASM** is a web project that compiles the dll into a WASM file. It is the main user interface.
* **MathParserWPF** a Windows desktop GUI for the parser dll. 
* **MathParserLib** C# version. [![deprecated](http://badges.github.io/stability-badges/dist/deprecated.svg)](http://github.com/badges/stability-badges)
* **MathParserLibTests** C# tests. [![deprecated](http://badges.github.io/stability-badges/dist/deprecated.svg)](http://github.com/badges/stability-badges)

Those who are into parser development may want to take a look at the `.ebnf` file (which stands for *Erik's BNF* :))

## Build stack
All projects in **Visual Studio**, except for the WASM project, which is build in **VS Code**.
The WASM compilation obviously uses `emscripten`.
In addition `rollup` is used to bundle the CodeMirror 6 online editor.
Note that a separate parser is written for CodeMirror syntax highlighting.
The (json) output of the C++ parser is fed to CodeMirror's *linter* for error annotation.
## Try it
There's a online version available at [Google Cloud Platform](https://storage.googleapis.com/mathparser/index.html).

## Disclaimer
This is a personal project, for fun.  
Don't let your rocket launches depend on it.

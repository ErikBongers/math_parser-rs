use math_parser::test_api::test_exponent;
use math_parser::test_api::{test_compiles, test_result, test_error, test_date, test_no_error};
use math_parser::errors::ErrorId;

#[test]
fn test_numbers (){
    test_result("123.456", 123.456, "");
    test_result("0.123", 0.123, "");
    test_result("-1", -1.0, "");
}
#[test]
fn test_constants (){
    test_result("trunc(PI)", 3.0, "");
}

#[test]
fn test_dates (){
    test_date("#define dmy \n date(1,2,2022)", 1, 2, 2022);
    test_error("#define ymd \n date(1,2,2022)", ErrorId::InvDate);
    test_date("#define dmy \n '1-2-2022'", 1, 2, 2022);
    test_error("#define ymd \n '1-2-2022'", ErrorId::InvDateStrForFormat);
}

#[test]
fn test_mute (){
    test_result("a=1;#a=2", 1.0, ""); //since the last result is muted, thus not returned.
    test_result("a=1;/#a=2;a=3;a=4;", 1.0, "");
    test_result("a=1;/#a=2;a=3;a=4;#/a=5", 5.0, "");
}

#[test]
fn test_simple_expr (){
    test_result("1+2", 3.0, "");
    test_result("2*3", 6.0, "");
    test_result("6/3", 2.0, "");
    test_result("2^3", 8.0, "");
    test_result("|-8|", 8.0, "");
    test_result("10%12", 10.0, "");
    test_result("-10%12", -10.0, "");
    test_result("-10%%12", 2.0, "");

    test_result("0!", 1.0, "");
    test_result("1!", 1.0, "");
    test_result("2!", 2.0, "");
    test_result("5!", 120.0, "");
    test_error("5.3!", ErrorId::ValueError);
    test_error("(-5)!", ErrorId::ValueError);
    test_error("21!", ErrorId::ValueError);

    test_result("2*2*3", 12.0, "");
    test_result("7-2*3", 1.0, "");
    test_result("7-(2*3)", 1.0, "");
    test_result("(7-2)*3", 15.0, "");
    test_result("1*2+3*4+5*6", 44.0, "");
}

#[test]
fn test_precedence () {
    test_result("1+2*3^4;", 163.0, "");
    test_result("4-3-2+1;", 0.0, "");
    test_result("12/3*2;", 8.0, "");
}

#[test]
fn test_implicit_mult () {
    test_result("a=2;2a", 4.0, "");
    test_result("a=2;(2)a", 4.0, "");
    test_result("a=2;20/2a", 5.0, "");
    test_result("a=2; 3 a;", 6.0, "");
    test_result("a=2; 3(4a);", 24.0, "");
    test_result("a=2; a(3(4a));", 48.0, "");
    test_result("m=2; 2m;", 4.0, "");
    test_error("m=2; 2m;", ErrorId::WUnitIsVar);
}

#[test]
fn test_formatted_number () {
    test_result("a='123", 123.0, ""); //string is not closed before EOS: no error!
    test_result("a='123,456.789'", 123456.789, "");
    test_result("a='123.456,789'", 123456.789, "");
    test_result("a='123456789'", 123456789.0, "");
    test_result("#define decimal_dot \n a='123,456.789'", 123456.789, "");
    test_result("#define decimal_comma \n a='123.456,789'", 123456.789, "");
    test_result("#define decimal_dot\n a='.123'", 0.123, "");
    test_result("a='12,34,56.789'", 123456.789, "");
    test_result("a='12.34.56,789'", 123456.789, "");
    test_error("a='12.34,56.789'", ErrorId::InvNumberStr);
    test_error("a='12,34.56,789'", ErrorId::InvNumberStr);
    test_error("a='12,3456789'", ErrorId::InvNumberStr);
    test_error("a='12.3456789'", ErrorId::InvNumberStr);
}

#[test]
fn test_assign_expr () {
    test_result("a=1;b=2;c=a+b", 3.0, "");
    test_result("a=1;a+=2", 3.0, "");
    test_result("a=1mm;a.=", 1.0, "");
    test_result("a=1;a=2;a;", 2.0, "");
    test_result("a=2;a*=3;", 6.0, "");
    test_result("a=6;a/=3;", 2.0, "");
    test_result("a=1;a+=3;", 4.0, "");
    test_result("a=5;a-=3;", 2.0, "");
    test_result("  a=3;a.=km               ", 3.0, "km");
    test_result("  a=3km; a.=;             ", 3.0, "");
    test_result("  a=3;a*=2                ", 6.0, "");
    test_result("  a=3;a*=2km              ", 6.0, "km");
    test_result("  a=3km; a*=2;            ", 6.0, "km");
    test_result("  a=3;a+=2                ", 5.0, "");
    test_error("  a=3km; a+=2;            ", ErrorId::WAssumingUnit);
    test_error("  a=3; a+=2km;            ", ErrorId::WAssumingUnit);
    test_result("  dec=123;                ", 123.0, ""); // dec should not conflict with the dec() function.
}

#[test]
fn test_global_funcs () {
    test_result("abs(-1)", 1.0, "");
    test_result("abs(0-123)", 123.0, "");
    test_result("a=1; a++", 2.0, "");
    test_result("a=2; a--", 1.0, "");
    test_result("sum(1,2,3)", 6.0, "");
    // test_result("sum(1,2, now())", 6.0, "");
    test_result("max(1,2,3)", 3.0, "");
    test_result("min(1,2,3)", 1.0, "");
    test_result("avg(1,2,3)", 2.0, "");
    test_result("first(1,2,3)", 1.0, "");
    test_result("last(1,2,3)", 3.0, "");
    test_result("first(reverse(1,2,3))", 3.0, "");
    test_result("first(sort(3,1, 2))", 1.0, "");
    test_result("a=30deg; sin(a)", 0.5, "");
    test_compiles("now()");
}
#[test]
fn test_name_conflicts () {
    test_error("  km=123                    ", ErrorId::WVarIsUnit);
    test_error("  sin=123                   ", ErrorId::WVarIsFunction);
}

#[test]
fn test_unary () {
    test_result("a=-1;", -1.0, ""); //not really a unary, but for completeness sake
    test_result("a=-(1);", -1.0, "");
    test_result("a=-sin(30deg);", -0.5, "");
    test_result("a=1; a=-a;", -1.0, "");
}

#[test]
fn test_calls () {
    test_result("sin(30deg)", 0.5, "");
    test_result("sin(30deg)+1", 1.5, "");
    test_result("abs(-sin(30deg))", 0.5, "");
    test_result("|-sin(30deg)|", 0.5, "");
    test_result("a=-123;|a|", 123.0, "");
    test_result("max(-sin(30deg), 3, 9999)", 9999.0, "");
}

#[test]
fn test_functions () {
    test_error("factorial(-1);", ErrorId::ValueError);

    test_result("\
    a=3;
function hundred(a)
  {
  a*100;
  }; //semi-colon should be ignored.
hundred(a);
", 300.0, "");
    test_result("\
            //outer scope var not changed
a=3;
function hundred(a)
  {
  a*100;
  }
hundred(a);
a;
", 3.0, "");
    test_error("\
            //inner scope var not visible outside.
a=3;
function hundred(a)
  {
  x = 123;
  a*100;
  }
hundred(a);
x;

", ErrorId::VarNotDef);

    test_result("\
            //nested functions
a=3;
function hundred(a)
  {
  function getFactor(z) { z*5; }
  a*getFactor(20);
  }
hundred(a);
", 300.0, "");
    test_error("function doppel(x) {}; function doppel(x){}", ErrorId::WFunctionOverride);
    test_result("sum(1,2,3,4);", 10.0, "");
    //test recursive (nested) lists.
    test_result("lizt=1,2,3; sum(lizt,4);", 10.0, "");
    test_result("lizt=(1,(2,3),4); list2=(5, lizt); sum(1, list2,6);", 22.0, "");
    test_result("lizt=((5, (1,(2,3),4)), 1); sum(lizt);", 16.0, "");
    test_result("lizt=((5, (1,(2,3),4)), 1); first(flatten(lizt));", 5.0, "");
    test_result("lizt=((5, (1,(2,3),4)), 1); last(flatten(lizt));", 1.0, "");
    test_result("lizt=((5, (1,(2,3),4)), 3); avg(flatten(lizt));", 3.0, "");
    test_error("sum(1, now());", ErrorId::FuncArgWrongType);
}

#[test]
fn test_number_formats () {
    test_result("123.dec;", 123.0, "");
    test_result("123.hex;", 123.0, "");
    test_result("123.oct;", 123.0, "");
    test_result("123.bin;", 123.0, "");
    test_result("0o173", 123.0, "");
    test_result("0O173", 123.0, "");
    test_result("0x7b", 123.0, "");
    test_result("0X7B", 123.0, "");
    test_result("0b1111011", 123.0, "");
    test_result("0B1111011", 123.0, "");
    test_result("123.456", 123.456, "");
    test_result("'1,234.56'", 1234.56, "");
    test_result("#define decimal_comma \n '1.234,56'", 1234.56, "");
    test_result("123_456", 123456.0, "");
    test_result("-123_456", -123456.0, "");
    test_result("0b111_1011", 123.0, "");
    test_result("0x1E240", 123456.0, "");
    test_result("0b111_1011C.dec", 123.0, "C");
    test_result("(0b111_1011.)", 123.0, "");
    test_result("(0b111_1011C.).dec", 123.0, "");}

#[test]
fn test_units () {
    test_result("(10.3+3).m-300cm", 10.3, "m");
    test_result("1L", 1.0, "L");
    test_result("1L+100ml", 1.1, "L");
    test_result("(1m)mm", 1000.0, "mm");
    test_result("(1m).mm", 1000.0, "mm");
    test_result("(1.m)mm", 1000.0, "mm");
    test_result("1.m.mm", 1000.0, "mm");
    test_result("1m.mm", 1000.0, "mm");
    test_result("sum(1mm, 2cm)", 21.0, "mm");
    test_result("sum(1mm, 2cm).mm", 21.0, "mm");
    test_result("sum(1mm, 2cm)mm", 21.0, "mm");
    test_result("1mm.", 1.0, "");
    test_error("sin(1mm)", ErrorId::UnitPropWrong);
    test_no_error("sin(1)");
    test_error("#define strict\n sin(1)", ErrorId::WExplicitUnitsExpected);
    test_error("3mm+2ml", ErrorId::UnitPropDiff);
    test_result("  10km+5m                 ", 10.005, "km");
    test_result("  (10km+5m).              ", 10.005, "");
    test_result("  (10km.)                 ", 10.0, "");
    test_result("  (10km+5m).m             ", 10005.0, "m");
    test_result("  10kg.g.N.lb.kg          ", 10.0, "kg");
    test_result("  max(10,20)km            ", 20.0, "km");
    test_result("  max(10m,2000m)km        ", 2.0, "km");
    test_result("  a=10;a.km               ", 10.0, "km");
    test_result("  a=10;(a).km             ", 10.0, "km");
    test_result("  a=10;(a)km              ", 10.0, "km");
    test_error("   a=10g;(a)km             ", ErrorId::UnitPropDiff);
    test_error("   a=10g;a.km              ", ErrorId::UnitPropDiff);
    test_result("  a=10g;(a.)km            ", 10.0, "km");
    test_error("   max(1kg, 4C)            ", ErrorId::UnitPropDiff);
    test_result("  max(1kg, 4g)            ", 1.0, "kg");
    test_result("  (PI)kg                  ", 1.0 + 2.1415927, "kg"); //rust compiler recognizes PI value, hence the 1+2.14...
    test_result("  1m*2                    ", 2.0, "m");
    test_result("  1*2m                    ", 2.0, "m");
    test_result("  a=2; a.=m;              ", 2.0, "m");
    test_error("   a=1; a b;               ", ErrorId::UnitNotDef);
    test_result("  a=2000mm; meter = 1m; a.=meter; ", 2.0, "m");
    test_error("   2asdfsdfsdf               ", ErrorId::UnitNotDef);
}

#[test]
fn test_id_names () {
    test_result("  a=3;                    ", 3.0, "");
    test_result("  a_b=3;                  ", 3.0, "");
    test_result("  _a_b=3;                 ", 3.0, "");
    test_result("  Δa=3;                 ", 3.0, "");
}

#[test]
fn test_nonsense () {
    //just test if it doesn't crash.
    test_compiles("");
    test_compiles(";");
    test_compiles("-");
    test_compiles("date(2022, 'sdf', 31)"); //fales in c++!
    test_compiles("a=1; { a=2; "); //block not closed.
    test_compiles("a=");
}

#[test]
fn test_defines () {
    test_error("#define short_date_units \n s=1;", ErrorId::WVarIsUnit);
    test_error("#undef trig\n  sin(1);", ErrorId::FuncNotAccessible);
    test_result("#undef trig\n#define trig\n  sin(30deg);", 0.5, "");
    test_error("#undef date\n  now();", ErrorId::FuncNotAccessible);
    //strict and constants
    test_error("#define strict\n  PI=1;", ErrorId::ConstRedef);
    test_error("PI=1;", ErrorId::WConstRedef);
    test_result("PI=1;", 1.0, "");

    test_error("#define precision=1.2", ErrorId::Expected);
    test_no_error("#define precision=2");
    test_no_error("#undef date\n#define date\n  now();");
}
#[test]
fn test_strict () {
    test_error("#define strict\n  function ff(a) { a+1; } function ff(b) { b+1; } ", ErrorId::FunctionOverride);
    test_error("function ff(a) { a+1; } function ff(b) { b+1; } ", ErrorId::WFunctionOverride);
    test_error("mm=23", ErrorId::WVarIsUnit);
    test_error("#define strict\n  mm=23", ErrorId::VarIsUnit);

}

#[test]
fn test_exponents () {
    test_exponent("123E10 + 234E10", 357.0, "", 10);
    test_exponent("123E10 + 2340E9", 357.0, "", 10);
    test_exponent("234E10 - 123E10", 111.0, "", 10);
    test_exponent("123E10 * 2E10", 246.0, "", 20);
    test_exponent("123E10 * 20E9", 2460.0, "", 19);
    test_exponent("246E10 / 2E10", 123.0, "", 0);
    test_exponent("1E-2", 1.0, "", -2);
    //combined with units:
    test_exponent("1E2mm.cm", 0.1, "cm", 2);
    test_exponent("1E3m + 1km", 2.0, "m", 3);
    test_exponent("1E5cm + 1km", 2.0, "cm", 5);
}

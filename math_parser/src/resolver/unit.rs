use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use crate::resolver::globals::Globals;
use crate::tokenizer::cursor::Range;

#[derive(Clone)]
pub struct Unit {
    pub range: Option<Range>,
    pub id: String,
}
impl Unit {
    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }

    pub fn none() -> Unit {
        Unit {
            range: None,
            id: "".to_string()
        }
    }

    pub fn from_id(id: &str) -> Unit {
        Unit {
            range: None,
            id: id.to_string(),
        }
    }
}

#[derive(PartialEq)]
pub enum UnitProperty { ANGLE, LENGTH, TEMP, MassWeight, DURATION, VOLUME, CURRENT, VOLTAGE, RESISTANCE, UNDEFINED }

pub struct UnitDef {
    pub to_si_factor: f64,
    pub id: String,
    pub si_id: &'static str,
    pub property: UnitProperty,
    pub to_si: fn(&UnitDef, f64) -> f64, //TODO: see if this works with self instead of explicit ref to UnitDef
    pub from_si: fn(&UnitDef, f64) -> f64,
}

#[derive(Clone)]
pub struct UnitsView {
    pub units: HashSet<String>
}

impl UnitsView {
    pub fn new(globals: &Globals) -> Self {
        let mut v = UnitsView {
            units: HashSet::new(),
        };
        v.add_all_classes(&globals.unit_defs, globals);
        v
    }

    pub fn get_def<'a>(&'a self, id: &str, globals: &'a Globals) -> Option<&UnitDef> {
        if self.units.contains(id) {
            return Some(&globals.unit_defs[id])
        }
        None
    }

    pub fn add_class(&mut self, property: &UnitProperty, globals: &Globals)
    {
        self.units.extend(globals.unit_defs
            .values()
            .filter(|unit| &unit.property == property )
            .map(|unit| unit.id.to_string())
        );
    }

    pub fn remove_class(&mut self, property: &UnitProperty, globals: &Globals) {
        self.units.retain(|unit| &globals.unit_defs[unit].property != property);
    }

    pub fn add_all_classes(&mut self, defs: &HashMap<String, UnitDef>, globals: &Globals) {
        self.add_class(&UnitProperty::UNDEFINED, globals); //needed to include the empty unit.
        self.add_class(&UnitProperty::ANGLE, globals);
        self.add_class(&UnitProperty::LENGTH, globals);
        self.add_class(&UnitProperty::TEMP, globals);
        self.add_class(&UnitProperty::MassWeight, globals);
        self.add_class(&UnitProperty::DURATION, globals);
        self.add_class(&UnitProperty::VOLUME, globals);
        //TODO: electricity
    }
}

pub fn default_to_si(def: &UnitDef, from: f64) -> f64 {
    from * def.to_si_factor
}
pub fn default_from_si(def: &UnitDef, from: f64) -> f64 {
    from / def.to_si_factor
}

pub fn create_unit_defs() -> HashMap<String, UnitDef> {

    let defs: HashMap<String, UnitDef> = HashMap::from( [
        ("".to_string(), UnitDef {id: "".to_string(), si_id: "", to_si_factor: 1.0, property: UnitProperty::UNDEFINED, from_si: default_from_si, to_si: default_to_si}),
        ( "rad".to_string(), UnitDef { id: "rad".to_string(), si_id: "rad", to_si_factor: 1.0, property: UnitProperty::ANGLE, from_si: default_from_si, to_si: default_to_si}),
        ( "deg".to_string(), UnitDef { id: "deg".to_string(), si_id: "rad", to_si_factor: PI / 180.0, property: UnitProperty::ANGLE, from_si: default_from_si, to_si: default_to_si}),

        ( "m".to_string(), UnitDef { id: "m".to_string(), si_id: "m", to_si_factor: 1.0, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "cm".to_string(), UnitDef { id: "cm".to_string(), si_id: "m", to_si_factor: 0.01, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "km".to_string(), UnitDef { id: "km".to_string(), si_id: "m", to_si_factor: 1000.0, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "mm".to_string(), UnitDef { id: "mm".to_string(), si_id: "m", to_si_factor: 0.001, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "in".to_string(), UnitDef { id: "in".to_string(), si_id: "m", to_si_factor: 0.0254, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "ft".to_string(), UnitDef { id: "ft".to_string(), si_id: "m", to_si_factor: 0.3048, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "thou".to_string(), UnitDef { id: "thou".to_string(), si_id: "m", to_si_factor: 0.0254 / 1000.0, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "yd".to_string(), UnitDef { id: "yd".to_string(), si_id: "m", to_si_factor: 0.9144, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "mi".to_string(), UnitDef { id: "mi".to_string(), si_id: "m", to_si_factor: 1609.344, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "micron".to_string(), UnitDef { id: "micron".to_string(), si_id: "m", to_si_factor: 0.000001, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "um".to_string(), UnitDef { id: "um".to_string(), si_id: "m", to_si_factor: 0.000001, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "ly".to_string(), UnitDef { id: "ly".to_string(), si_id: "m", to_si_factor: 9460730472580800.0, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),

        ( "C".to_string(), UnitDef { id: "C".to_string(), si_id: "K", to_si_factor: 0.000001, property: UnitProperty::TEMP, from_si: default_from_si, to_si: default_to_si}),
        ( "K".to_string(), UnitDef { id: "K".to_string(), si_id: "K", to_si_factor: 1.0, property: UnitProperty::TEMP, from_si: default_from_si, to_si: default_to_si}),
        ( "F".to_string(), UnitDef { id: "F".to_string(), si_id: "K", to_si_factor: 0.000001, property: UnitProperty::TEMP, from_si: default_from_si, to_si: default_to_si}),

        ( "L".to_string(), UnitDef { id: "L".to_string(), si_id: "L", to_si_factor: 1.0, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),
        ( "mL".to_string(), UnitDef { id: "mL".to_string(), si_id: "L", to_si_factor: 0.001, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),
        //ml, with lower case l is non standard
        ( "ml".to_string(), UnitDef { id: "ml".to_string(), si_id: "L", to_si_factor: 0.001, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),
        ( "gal".to_string(), UnitDef { id: "gal".to_string(), si_id: "L", to_si_factor: 3.785411784, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),
        ( "pt".to_string(), UnitDef { id: "pt".to_string(), si_id: "L", to_si_factor: 0.473176473, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),

        ( "kg".to_string(), UnitDef { id: "kg".to_string(), si_id: "kg", to_si_factor: 1.0, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "N".to_string(), UnitDef { id: "N".to_string(), si_id: "kg", to_si_factor: 1.0/9.80665, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "g".to_string(), UnitDef { id: "g".to_string(), si_id: "kg", to_si_factor: 0.001, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "mg".to_string(), UnitDef { id: "mg".to_string(), si_id: "kg", to_si_factor: 0.000001, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "t".to_string(), UnitDef { id: "t".to_string(), si_id: "kg", to_si_factor: 1000.0, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "lb".to_string(), UnitDef { id: "lb".to_string(), si_id: "kg", to_si_factor: 0.45359, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "lbs".to_string(), UnitDef { id: "lbs".to_string(), si_id: "kg", to_si_factor: 0.45359, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "oz".to_string(), UnitDef { id: "oz".to_string(), si_id: "kg", to_si_factor: 1.0/ 35.2739619496, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),

        ( "seconds".to_string(), UnitDef { id: "seconds".to_string(), si_id: "s", to_si_factor: 1.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "minutes".to_string(), UnitDef { id: "minutes".to_string(), si_id: "s", to_si_factor: 60.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "hours".to_string(), UnitDef { id: "hours".to_string(), si_id: "s", to_si_factor: 3600.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "days".to_string(), UnitDef { id: "days".to_string(), si_id: "s", to_si_factor: 86400.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "weeks".to_string(), UnitDef { id: "weeks".to_string(), si_id: "s", to_si_factor: (60 * 60 * 24 * 7) as f64, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "months".to_string(), UnitDef { id: "months".to_string(), si_id: "s", to_si_factor: 2629746.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "years".to_string(), UnitDef { id: "years".to_string(), si_id: "s", to_si_factor: 31556952.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "milliseconds".to_string(), UnitDef { id: "milliseconds".to_string(), si_id: "s", to_si_factor: 1.0/1000.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),

        ( "s".to_string(), UnitDef { id: "s".to_string(), si_id: "s", to_si_factor: 1.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "min".to_string(), UnitDef { id: "min".to_string(), si_id: "s", to_si_factor: 60.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "h".to_string(), UnitDef { id: "h".to_string(), si_id: "s", to_si_factor: 3600.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "d".to_string(), UnitDef { id: "d".to_string(), si_id: "s", to_si_factor: 86400.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "w".to_string(), UnitDef { id: "w".to_string(), si_id: "s", to_si_factor: (60 * 60 * 24 * 7) as f64, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "mon".to_string(), UnitDef { id: "mon".to_string(), si_id: "s", to_si_factor: 2629746.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "y".to_string(), UnitDef { id: "y".to_string(), si_id: "s", to_si_factor: 31556952.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "ms".to_string(), UnitDef { id: "ms".to_string(), si_id: "s", to_si_factor: 1.0/1000.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),

        ( "A".to_string(), UnitDef { id: "A".to_string(), si_id: "A", to_si_factor: 1.0, property: UnitProperty::CURRENT, from_si: default_from_si, to_si: default_to_si}),
        ( "mA".to_string(), UnitDef { id: "mA".to_string(), si_id: "A", to_si_factor: 0.001, property: UnitProperty::CURRENT, from_si: default_from_si, to_si: default_to_si}),

        ( "R".to_string(), UnitDef { id: "R".to_string(), si_id: "R", to_si_factor: 1.0, property: UnitProperty::RESISTANCE, from_si: default_from_si, to_si: default_to_si}),
        ( "mR".to_string(), UnitDef { id: "mR".to_string(), si_id: "R", to_si_factor: 0.001, property: UnitProperty::RESISTANCE, from_si: default_from_si, to_si: default_to_si}),
        ( "kR".to_string(), UnitDef { id: "kR".to_string(), si_id: "R", to_si_factor: 1000.0, property: UnitProperty::RESISTANCE, from_si: default_from_si, to_si: default_to_si}),
        ( "MR".to_string(), UnitDef { id: "MR".to_string(), si_id: "R", to_si_factor: 1000000.0, property: UnitProperty::RESISTANCE, from_si: default_from_si, to_si: default_to_si}),

        ( "V".to_string(), UnitDef { id: "V".to_string(), si_id: "V", to_si_factor: 1.0, property: UnitProperty::VOLTAGE, from_si: default_from_si, to_si: default_to_si}),
        ( "mV".to_string(), UnitDef { id: "mV".to_string(), si_id: "V", to_si_factor: 0.001, property: UnitProperty::VOLTAGE, from_si: default_from_si, to_si: default_to_si}),

    ]);
    defs
}

#[cfg(test)]
mod tests {
    use crate::resolver::globals::Globals;
    use crate::resolver::unit::{create_unit_defs, UnitProperty, UnitsView};

    #[test]
    fn test_units() {
        let globals = Globals::new();
        let mut view = UnitsView::new(&globals);
        view.units.clear();
        view.add_class(&UnitProperty::ANGLE, &globals);
        assert_eq!(view.units.len(), 2);
    }

    #[test]
    fn test_clone_units() {
        let globals = Globals::new();
        let mut view = UnitsView::new(&globals);
        view.units.clear();
        view.add_class(&UnitProperty::ANGLE, &globals);
        view.add_class(&UnitProperty::TEMP, &globals);
        assert_eq!(view.units.len(), 5);

        view.remove_class(&UnitProperty::ANGLE, &globals);
        assert_eq!(view.units.len(), 3);
    }
}

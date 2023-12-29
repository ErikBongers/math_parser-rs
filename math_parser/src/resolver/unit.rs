use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
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
}

#[derive(PartialEq)]
pub enum UnitProperty { ANGLE, LENGTH, TEMP, MassWeight, DURATION, VOLUME, CURRENT, VOLTAGE, RESISTANCE, UNDEFINED }

pub struct UnitDef<'a> {
    to_si_factor: f64,
    id: &'a str,
    pub property: UnitProperty,
    pub to_si: fn(&UnitDef, f64) -> f64, //TODO: see if this works with self instead of explicit ref to UnitDef
    pub from_si: fn(&UnitDef, f64) -> f64,
}

pub struct UnitsView<'a> {
    pub units: HashSet<&'a str>
}

impl<'a> UnitsView<'a> {
    pub fn new() -> Self {
        UnitsView {
            units: HashSet::new()
        }
    }

    pub fn get_def<'b>(&self, id: &str, defs: &'b HashMap<&'a str, UnitDef<'a>>) -> Option<&'b UnitDef> {
        if self.units.contains(id) {
            return Some(&defs[id])
        }
        None
    }

    pub fn add_class(&mut self, property: &UnitProperty, defs: &HashMap<&'a str, UnitDef<'a>>)
    {
        self.units.extend(defs
            .values()
            .filter(|unit| &unit.property == property )
            .map(|unit| &unit.id)
        );
    }

    pub fn remove_class(&mut self, property: &UnitProperty, defs: &HashMap<&'a str, UnitDef<'a>>) {
        self.units.retain(|unit| &defs[unit].property != property);
    }

    pub fn add_all_classes(&mut self, defs: &HashMap<&'a str, UnitDef<'a>>) {
        self.add_class(&UnitProperty::UNDEFINED, defs); //needed to include the empty unit.
        self.add_class(&UnitProperty::ANGLE, defs);
        self.add_class(&UnitProperty::LENGTH, defs);
        self.add_class(&UnitProperty::TEMP, defs);
        self.add_class(&UnitProperty::MassWeight, defs);
        self.add_class(&UnitProperty::DURATION, defs);
        self.add_class(&UnitProperty::VOLUME, defs);
        //TODO: electricity
    }
}

pub fn default_to_si(def: &UnitDef, from: f64) -> f64 {
    from * def.to_si_factor
}
pub fn default_from_si(def: &UnitDef, from: f64) -> f64 {
    from / def.to_si_factor
}

pub fn create_unit_defs<'a>() -> HashMap<&'a str, UnitDef<'a>> {

    let defs: HashMap<&str, UnitDef> = HashMap::from( [
        ("", UnitDef {id: "", to_si_factor: 1.0, property: UnitProperty::UNDEFINED, from_si: default_from_si, to_si: default_to_si}),
        ( "rad", UnitDef { id: "rad", to_si_factor: 1.0, property: UnitProperty::ANGLE, from_si: default_from_si, to_si: default_to_si}),
        ( "deg", UnitDef { id: "deg", to_si_factor: PI / 180.0, property: UnitProperty::ANGLE, from_si: default_from_si, to_si: default_to_si}),

        ( "m", UnitDef { id: "m", to_si_factor: 1.0, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "cm", UnitDef { id: "cm", to_si_factor: 0.01, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "km", UnitDef { id: "km", to_si_factor: 1000.0, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "mm", UnitDef { id: "mm", to_si_factor: 0.001, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "in", UnitDef { id: "in", to_si_factor: 0.0254, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "ft", UnitDef { id: "ft", to_si_factor: 0.3048, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "thou", UnitDef { id: "thou", to_si_factor: 0.0254 / 1000.0, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "yd", UnitDef { id: "yd", to_si_factor: 0.9144, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "mi", UnitDef { id: "mi", to_si_factor: 1609.344, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "micron", UnitDef { id: "micron", to_si_factor: 0.000001, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "um", UnitDef { id: "um", to_si_factor: 0.000001, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),
        ( "ly", UnitDef { id: "ly", to_si_factor: 9460730472580800.0, property: UnitProperty::LENGTH, from_si: default_from_si, to_si: default_to_si}),

        ( "C", UnitDef { id: "C", to_si_factor: 0.000001, property: UnitProperty::TEMP, from_si: default_from_si, to_si: default_to_si}),
        ( "K", UnitDef { id: "K", to_si_factor: 1.0, property: UnitProperty::TEMP, from_si: default_from_si, to_si: default_to_si}),
        ( "F", UnitDef { id: "F", to_si_factor: 0.000001, property: UnitProperty::TEMP, from_si: default_from_si, to_si: default_to_si}),

        ( "L", UnitDef { id: "L", to_si_factor: 1.0, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),
        ( "mL", UnitDef { id: "mL", to_si_factor: 0.001, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),
        //ml, with lower case l is non standard
        ( "ml", UnitDef { id: "ml", to_si_factor: 0.001, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),
        ( "gal", UnitDef { id: "gal", to_si_factor: 3.785411784, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),
        ( "pt", UnitDef { id: "pt", to_si_factor: 0.473176473, property: UnitProperty::VOLUME, from_si: default_from_si, to_si: default_to_si}),

        ( "kg", UnitDef { id: "kg", to_si_factor: 1.0, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "N", UnitDef { id: "N", to_si_factor: 1.0/9.80665, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "g", UnitDef { id: "g", to_si_factor: 0.001, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "mg", UnitDef { id: "mg", to_si_factor: 0.000001, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "t", UnitDef { id: "t", to_si_factor: 1000.0, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "lb", UnitDef { id: "lb", to_si_factor: 0.45359, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "lbs", UnitDef { id: "lbs", to_si_factor: 0.45359, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),
        ( "oz", UnitDef { id: "oz", to_si_factor: 1.0/ 35.2739619496, property: UnitProperty::MassWeight, from_si: default_from_si, to_si: default_to_si}),

        ( "seconds", UnitDef { id: "seconds", to_si_factor: 1.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "minutes", UnitDef { id: "minutes", to_si_factor: 60.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "hours", UnitDef { id: "hours", to_si_factor: 3600.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "days", UnitDef { id: "days", to_si_factor: 86400.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "weeks", UnitDef { id: "weeks", to_si_factor: (60 * 60 * 24 * 7) as f64, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "months", UnitDef { id: "months", to_si_factor: 2629746.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "years", UnitDef { id: "years", to_si_factor: 31556952.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "milliseconds", UnitDef { id: "milliseconds", to_si_factor: 1.0/1000.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),

        ( "s", UnitDef { id: "s", to_si_factor: 1.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "min", UnitDef { id: "min", to_si_factor: 60.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "h", UnitDef { id: "h", to_si_factor: 3600.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "d", UnitDef { id: "d", to_si_factor: 86400.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "w", UnitDef { id: "w", to_si_factor: (60 * 60 * 24 * 7) as f64, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "mon", UnitDef { id: "mon", to_si_factor: 2629746.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "y", UnitDef { id: "y", to_si_factor: 31556952.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),
        ( "ms", UnitDef { id: "ms", to_si_factor: 1.0/1000.0, property: UnitProperty::DURATION, from_si: default_from_si, to_si: default_to_si}),

        ( "A", UnitDef { id: "A", to_si_factor: 1.0, property: UnitProperty::CURRENT, from_si: default_from_si, to_si: default_to_si}),
        ( "mA", UnitDef { id: "mA", to_si_factor: 0.001, property: UnitProperty::CURRENT, from_si: default_from_si, to_si: default_to_si}),

        ( "R", UnitDef { id: "R", to_si_factor: 1.0, property: UnitProperty::RESISTANCE, from_si: default_from_si, to_si: default_to_si}),
        ( "mR", UnitDef { id: "mR", to_si_factor: 0.001, property: UnitProperty::RESISTANCE, from_si: default_from_si, to_si: default_to_si}),
        ( "kR", UnitDef { id: "kR", to_si_factor: 1000.0, property: UnitProperty::RESISTANCE, from_si: default_from_si, to_si: default_to_si}),
        ( "MR", UnitDef { id: "MR", to_si_factor: 1000000.0, property: UnitProperty::RESISTANCE, from_si: default_from_si, to_si: default_to_si}),

        ( "V", UnitDef { id: "V", to_si_factor: 1.0, property: UnitProperty::VOLTAGE, from_si: default_from_si, to_si: default_to_si}),
        ( "mV", UnitDef { id: "mV", to_si_factor: 0.001, property: UnitProperty::VOLTAGE, from_si: default_from_si, to_si: default_to_si}),

    ]);
    defs
}

#[cfg(test)]
mod tests {
    use crate::resolver::unit::{create_unit_defs, UnitProperty, UnitsView};

    #[test]
    fn test_units() {
        let defs = create_unit_defs();
        let mut view = UnitsView::new();
        view.add_class(&UnitProperty::ANGLE, &defs);
        assert_eq!(view.units.len(), 2);
    }

    #[test]
    fn test_clone_units() {
        let defs = create_unit_defs();
        let mut view = UnitsView::new();
        view.add_class(&UnitProperty::ANGLE, &defs);
        view.add_class(&UnitProperty::TEMP, &defs);
        assert_eq!(view.units.len(), 5);

        view.remove_class(&UnitProperty::ANGLE, &defs);
        assert_eq!(view.units.len(), 3);
    }
}

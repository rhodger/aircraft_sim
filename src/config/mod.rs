//! Handles configuration such as reading from files and generating boarding
//! patterns.

use std::fs::File;
use std::path::Path;

use super::aircraft::Aircraft;
use super::aircraft::tile::Variant;
use super::aircraft::person::Person;

use rand::thread_rng;
use rand::seq::SliceRandom;

/// Temporary holder for data about seats when reading from files.
struct seat_data {
    seat_x: u16,
    seat_y: u16,
    variant: Variant,
}

impl seat_data {
    /// Constructor
    fn new(x: u16, y: u16, variant: &str) -> seat_data {
        seat_data {
            seat_x: x,
            seat_y: y,
            variant: str_to_var(variant),
        }
    }

    fn get_x(&self) -> u16 {
        self.seat_x
    }

    fn get_y(&self) -> u16 {
        self.seat_y
    }

    fn get_variant(&self) -> Variant {
        self.variant
    }
}

/// Converts a string into an associated Variant.
///
/// This is for use when calling Rust code from Python, as Python does not share
/// the same enums.
fn str_to_var(var: &str) -> Variant {
    match var {
        "aisle" => Variant::Aisle,
        "seat" => Variant::Seat,
        "entrance" => Variant::Entrance,
        _ => Variant::None, // Shouldn't happen
    }
}

/// Generates an aircraft with a single aisle down the center and seats either
/// side in the given size.
pub fn standard_layout(mut size_x: u16, size_y: u16)
    -> Result<Aircraft, &'static str> {

    log::info!("Generating standard-layout aircraft");

    if size_x % 2 == 0 {
        log::warn!("Invalid size_x value; assuming closest odd value.");
        size_x += 1;
    }


    let mut aircraft = Aircraft::new(size_x, size_y);
    let aisle = size_x / 2;
    for y in 0..size_y {
        for x in 0..size_x {
            if x != aisle {
                aircraft.set_tile(x, y, Variant::Seat);
            }
        }
    }
    aircraft.set_tile(aisle, size_y - 1, Variant::Entrance);

    return Ok(aircraft);
}

/// Generates a list of passengers that will board in standard back-first order
/// with randomised positions on each row.
pub fn random_back_first(mut size_x: u16, size_y: u16)
    -> Result<Vec<Person>, &'static str> {

    log::info!("Generating random back-first boarding pattern");
    if size_x % 2 == 0 {
        log::warn!("Invalid size_x value; assuming closest odd value.");
        size_x += 1;
    }
    let mut persons = Vec::<Person>::new();
    let aisle: u16 = size_x / 2;

    for y in 0..size_y {
        let mut x_coords: Vec<u16> = (0..size_x).collect();
        x_coords.shuffle(&mut thread_rng());
        for x in x_coords {
            if x != aisle {
                let mut person = Person::new("DEFAULT");
                person.target_seat(x, size_y - (y + 1));
                person.set_baggage(true);
                persons.push(person);
            }
        }
    }

    Ok(persons)
}

/// Generates a list of passengers that will board in standard front-first order
/// with randomised positions on each row.
pub fn random_front_first(mut size_x: u16, size_y: u16)
    -> Result<Vec<Person>, &'static str> {

    log::info!("Generating random front-first boarding pattern");
    if size_x % 2 == 0 {
        log::warn!("Invalid size_x value; assuming closest odd value.");
        size_x += 1;
    }
    let mut persons = Vec::<Person>::new();
    let aisle: u16 = size_x / 2;

    for y in 0..size_y {
        let mut x_coords: Vec<u16> = (0..size_x).collect();
        x_coords.shuffle(&mut thread_rng());
        for x in x_coords {
            if x != aisle {
                let mut person = Person::new("DEFAULT");
                person.target_seat(x, y);
                person.set_baggage(true);
                persons.push(person);
            }
        }
    }

    Ok(persons)
}

/// Generates a list of passengers that will board in standard window-first
/// order with randomised positions on each row.
pub fn random_window_first(mut size_x: u16, size_y: u16)
    -> Result<Vec<Person>, &'static str> {

    log::info!("Generating random aisle-first boarding pattern");
    if size_x % 2 == 0 {
        log::warn!("Invalid size_x value; assuming closest odd value.");
        size_x += 1;
    }
    let mut persons = Vec::<Person>::new();
    let aisle: u16 = size_x / 2;

    for x in 0..aisle {
        let mut y_coords: Vec<u16> = (0..size_y).collect();
        y_coords.shuffle(&mut thread_rng());
        for y in y_coords {
            let mut person0 = Person::new("DEFAULT");
            let mut person1 = Person::new("DEFAULT");
            
            person0.target_seat((aisle - (x + 1)), y);
            person0.set_baggage(true);
            person1.target_seat(aisle + (x + 1), y);
            person1.set_baggage(true);
            
            persons.push(person0);
            persons.push(person1);
        }
    }

    Ok(persons)
}

/// Generates a list of passengers that will board in standard aisle-first
/// order with randomised positions on each row.
pub fn random_aisle_first(mut size_x: u16, size_y: u16)
    -> Result<Vec<Person>, &'static str> {

    log::info!("Generating random aisle-first boarding pattern");
    if size_x % 2 == 0 {
        log::warn!("Invalid size_x value; assuming closest odd value.");
        size_x += 1;
    }
    let mut persons = Vec::<Person>::new();
    let aisle: u16 = size_x / 2;

    for x in 0..aisle {
        let mut y_coords: Vec<u16> = (0..size_y).collect();
        y_coords.shuffle(&mut thread_rng());
        for y in y_coords {
            let mut person0 = Person::new("DEFAULT");
            let mut person1 = Person::new("DEFAULT");
            
            person0.target_seat(x, y);
            person0.set_baggage(true);
            person1.target_seat(size_x - (x + 1), y);
            person1.set_baggage(true);
            
            persons.push(person0);
            persons.push(person1);
        }
    }

    Ok(persons)
}

/// Generates a list of passengers that will board in completely random order.
pub fn random(mut size_x: u16, size_y: u16) -> Result<Vec<Person>, &'static str> {
    log::info!("Generating random aisle-first boarding pattern");
    if size_x % 2 == 0 {
        log::warn!("Invalid size_x value; assuming closest odd value.");
        size_x += 1;
    }
    let mut persons = Vec::<Person>::new();
    let aisle: u16 = size_x / 2;
    let mut coords: Vec<(u16,u16)> = Vec::new();

    for x in 0..size_x {
        if x != aisle {
            for y in 0..size_y {
                coords.push((x,y));
            }
        }
    }

    coords.shuffle(&mut thread_rng());

    for i in coords {
        let mut person = Person::new("DEFAULT");
        
        person.target_seat(i.0, i.1);
        person.set_baggage(true);
        
        persons.push(person);
    }

    Ok(persons)
}

/// Reads a list of passengers from a correctly formatted csv file and returns
/// them as a vector of `Person` objects.
pub fn read_passengers(path: &Path) -> Option<Vec<Person>> {
    let mut persons = Vec::<Person>::new();
    let file = File::open(path).expect("Invalid file path");
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        let record = result.expect("Result is invalid record");
        let mut data = Person::new(
            &record[0],
        );
        data.target_seat(record[1].parse().expect("Invalid x coord"),
                         record[2].parse().expect("Invalid y coord"));
        data.set_baggage(match &record[3] {
            "0" => false,
            "1" => true,
            _ => {
                log::error!("Invalid baggage value, defaulting to 0");
                false
            },
        });
        persons.push(data);
    }
    return Some(persons);
}

/// Reads a list of tiles from a correctly formatted csv and returns them as an
/// `Aircraft` object with that layout.
pub fn read_layout(path: &Path) -> Option<Aircraft> {
    let mut seats = Vec::<seat_data>::new();
    let file = File::open(path).expect("Invalid file path");
    let mut rdr = csv::Reader::from_reader(file);
    
    for result in rdr.records() {
        let record = result.expect("Result is invalid record");
        let data = seat_data::new(
            record[0].parse().expect("Couldn't parse"),
            record[1].parse().expect("Couldn't parse"),
            &record[2],
        );
        seats.push(data);
    }
    
    let (mut size_x, mut size_y) = (0, 0);
    for i in &seats {
        if i.get_x() > size_x { size_x = i.get_x() }
        if i.get_y() > size_y { size_y = i.get_y() }
    }
    size_x = size_x + 1;
    size_y = size_y + 1;

    let mut aircraft = Aircraft::new(size_x, size_y);
    for i in seats {
        aircraft.set_tile(i.get_x(), i.get_y(), i.get_variant());
    }

    return Some(aircraft);
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_read_passengers() {
    //     let persons = read_passengers(Path::new("./config/test_passengers.csv")).unwrap();

    //     assert_eq!(persons.len(), 2, "Incorrect number of records");

    //     let person = persons.get(0).expect("First record not found");

    //     assert_eq!(person.get_name(), "person0", "First record's name was wrong");
    //     assert_eq!(person.get_seat().unwrap().0.to_string(), "0", "First record's x coord was wrong");
    // }

    #[test]
    fn test_read_layout() {
        let aircraft = read_layout(Path::new("./config/test_layout.csv")).unwrap();

        assert_eq!((5,5), aircraft.get_size());
    }

    #[test]
    fn test_str_to_var() {
        assert_eq!(str_to_var("aisle"), Variant::Aisle);
        assert_eq!(str_to_var("seat"), Variant::Seat);
        assert_eq!(str_to_var("entrance"), Variant::Entrance);
        assert_eq!(str_to_var("none"), Variant::None);
        assert_eq!(str_to_var("invalid"), Variant::None);
    }

    #[test]
    fn rand_back_first() {
        let list = random_back_first(5, 10).unwrap();

        assert_eq!(list.len(), 40);
    }
}

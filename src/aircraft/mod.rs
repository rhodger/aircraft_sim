//! This module handles the creation and management of aircraft instances in
//! Rust.
//!
//! It also handles logging, as almost all other modules are called from one of
//! the structures defined here.

pub mod tile;
pub mod person;

use simple_logger::SimpleLogger;

use tile::{Tile, Variant};
use person::{Person, Behaviour};

const MAX_ITERATIONS: u16 = 1000;

/// A struct representing a simulated aircraft.
///
/// This object simulates an aircraft, consisting of `layout`, a 2d vector of
/// `Tile` objects representing each space onboard and its occupant(s) (or lack
/// thereof), a list of passengers waiting to board, the aircraft's size as a
/// convenient tuple, and a list of seats which should eventually be filled when
/// the aircraft has been fully boarded.
///
/// # Examples
///
/// ```
/// let aircraft = Aircraft::new(7,10);
///
/// assert_eq!((7,10), aircraft.get_size());
/// ```
pub struct Aircraft {
    size: (u16, u16),
    layout: Vec<Vec<Tile>>,
    passengers: Vec<Person>,
    targeted_seats: Vec<(u16,u16)>,
}

impl Aircraft {
    /// Constructor
    pub fn new(x: u16, y: u16) -> Aircraft {
        log::info!("Initialising aircraft with size {},{}", x, y);

        let mut aircraft = Aircraft {
            size: (x, y),
            layout: Vec::<Vec<Tile>>::new(),
            passengers: Vec::<Person>::new(),
            targeted_seats: Vec::<(u16,u16)>::new(),
        };
        aircraft.clear();
        return aircraft;
    }
    
    /// Resets every tile on an aircraft to an aisle
    fn clear(&mut self) {
        self.layout = Vec::<Vec<Tile>>::new();
        for i in 0..self.size.0 {
            self.layout.push(Vec::<Tile>::new());
            for _ in 0..self.size.1 {
                self.layout[i as usize].push(Tile::aisle());
            }
        }
    }
    
    /// Adds a passenger to the aircraft
    pub fn add_passenger(&mut self, p: Person) {
        let seat = p.get_seat();
        if seat.is_some() {
            self.targeted_seats.push(seat.unwrap());
        }
        self.passengers.push(p);
    }

    /// For debug purposes: prints the aircraft's layout to stdout using ASCII
    /// characters to represent passengers and different types of aisle.
    pub fn ascii_render(&self) {
        print!("   ");
        for i in 0..self.size.0 {
            print!("{}", i);
        }
        print!("\n");
        for j in 0..self.size.1 as usize {
            print!("{:>3}", j);
            for i in 0..self.size.0 as usize {
                if self.layout[i][j].is_occupied() {
                    print!("@");
                } else {
                    print!("{}", match self.layout[i][j].get_variant() {
                        Variant::Aisle => "*",
                        Variant::Seat => "#",
                        Variant::Entrance => "*",
                        Variant::None => "?",
                    });
                }
            }
            print!("\n");
        }
    }
    
    /// Determines the best move for a passenger.
    ///
    /// Takes into account the passenger's current location, their target
    /// location, whether they are currently holding baggage, and their
    /// surroundings to determine the best possible move at any given time. This
    /// is returned as a `Behaviour` enum.
    fn determine_move(&self,
                      i: usize,
                      j: usize,
                      target_x: u16,
                      target_y: u16,
                      baggage: bool) -> (Behaviour, f32) {
        let mut current_move = (Behaviour::Wait, 1000.0);
        let target_seat = (target_x, target_y);
        // let (i, j) = (pos_x as usize, pos_y as usize);
        
        if self.layout[i][j].get_variant() == Variant::Aisle
        || self.layout[i][j].get_variant() == Variant::Entrance {
            if target_seat.1 == j as u16 && baggage {
                current_move = (Behaviour::Stow, 0.0);
            } else {
                // Decide movement based on other tile
                for potential_move in &[
                    (Behaviour::Wait, (0.0, 0.0)),
                    (Behaviour::Move_North, (0.0, -1.0)),
                    (Behaviour::Move_South, (0.0, 1.0)),
                    (Behaviour::Move_East, (1.0, 0.0)),
                    (Behaviour::Move_West, (-1.0, 0.0)),
                ] {
                    let (dest_x, dest_y)
                        = (i as f32 + (potential_move.1).0,
                           j as f32 + (potential_move.1).1);
                    let (dest_x, dest_y)
                        = (dest_x as usize,
                           dest_y as usize);

                    // Calculates the distance from the potential tile to the
                    // target tile using the pythagorean theorum.
                    let new_distance
                        = ((target_seat.0 as f32 - dest_x as f32).abs()
                           + (target_seat.1 as f32 - dest_y as f32).abs());

                    if new_distance < current_move.1 {
                        // Check whether the movement would place this passenger
                        // on the wrong row and, if so, wait instead.
                        if !(potential_move.0 == Behaviour::Move_East
                             && dest_y != target_seat.1 as usize)
                            && !(potential_move.0 == Behaviour::Move_West
                                 && dest_y != target_seat.1 as usize) {
                            // Check whether desired seat is occupied
                            if !self.layout[dest_x][dest_y].is_occupied()
                                || (dest_x, dest_y) == (i, j) {
                                current_move = (potential_move.0, new_distance);
                                log::debug!("NEW MOVE: {:?} x {}",
                                            current_move.0,
                                            current_move.1);
                            } else if !self.layout[dest_x][dest_y].is_allowing()
                                && self.layout[dest_x][dest_y]
                                    .get_variant() == Variant::Seat {
                                // This movement is possible even though the
                                // position is occupied, as two passengers can
                                // temporarily share a space albeit with a
                                // delay.
                                current_move = (potential_move.0, new_distance);
                                log::debug!("NEW MOVE: {:?} x {}",
                                            current_move.0,
                                            current_move.1);
                            } else {
                                log::debug!("No room to get past");
                                log::debug!("REJECTED: {:?} x {}",
                                            potential_move.0,
                                            new_distance);
                            }
                        }
                    } else {
                        log::debug!("REJECTED: {:?} x {}",
                                    potential_move.0,
                                    new_distance);
                    }
                }
            }
        } else if self.layout[i][j].get_variant() == Variant::Seat {
            // Decide movement based on other tile
            for potential_move in &[
                (Behaviour::Wait, (0.0, 0.0)),
                (Behaviour::Move_East, (1.0, 0.0)),
                (Behaviour::Move_West, (-1.0, 0.0)),
            ] {
                let (dest_x, dest_y) = (i as f32 + (potential_move.1).0,
                                        j as f32 + (potential_move.1).1);
                // Calculates the distance from the potential tile to the target
                // tile using the pythagorean theorum.
                let new_distance = ((target_seat.0 as f32 - dest_x as f32).abs()
                                    + (target_seat.1 as f32 - dest_y as f32)
                                        .abs());

                if new_distance < current_move.1 {
                    current_move = (potential_move.0, new_distance);
                    log::debug!("NEW MOVE: {:?} x {}",
                                current_move.0,
                                current_move.1);
                } else {
                    log::debug!("REJECTED: {:?} x {}",
                                potential_move.0,
                                new_distance);
                }
            }
        }

        return current_move;
    }
    
    /// Updates every tile in the aircraft based on its current best move.
    ///
    /// Runs `determine_move` on every occupied tile in the aircraft, and, where
    /// possible, executes this move.
    pub fn update(&mut self) {
        for x in 0..self.size.0 as usize {
            for y in 0..self.size.1 as usize {
                // Check current tile variant
                    if !self.layout[x][y].has_updated() && (
                    self.layout[x][y].get_variant() == Variant::Entrance ||
                    self.layout[x][y].get_variant() == Variant::Aisle ||
                    self.layout[x][y].get_variant() == Variant::Seat ) {
                        // Check whether current tile is occupied
                        if self.layout[x][y].get_occupier().is_some() {
                            log::trace!("DEBUG: {:?}",
                                        self.layout[x][y]
                                            .get_occupier()
                                            .unwrap()
                                            .has_baggage());

                            // Choose movement
                            let target = self.layout[x][y].get_occupier()
                                .unwrap().get_seat().unwrap();
                            let baggage = self.layout[x][y].get_occupier()
                                .unwrap().has_baggage();
                            let current_move =
                                self.determine_move(x, y,
                                                    target.0, target.1,
                                                    baggage);

                            if current_move.0 != Behaviour::Wait
                            && current_move.0 != Behaviour::Stow {
                                log::debug!("Passenger moved: {:?}",
                                            current_move.0);

                                let coords = match current_move.0 {
                                    Behaviour::Move_North => (x, y - 1),
                                    Behaviour::Move_South => (x, y + 1),
                                    Behaviour::Move_East => (x + 1, y),
                                    Behaviour::Move_West => (x - 1, y),
                                    _ => {
                                        // Invalid move; wait instead
                                        log::warn!("Invalid move selected");
                                        (x, y) // May be slightly buggy, however
                                               // this branch shouldn't ever be
                                               // called
                                    },
                                };

                                if !self.layout[coords.0][coords.1]
                                  .is_occupied() {
                                    let person = self.layout[x][y].free();
                                    self.layout[coords.0][coords.1]
                                      .occupy(person.unwrap());
                                } else if !self.layout[coords.0][coords.1]
                                  .is_allowing() {
                                    let person = self.layout[x][y].free();
                                    self.layout[coords.0][coords.1]
                                      .pass_in(person.unwrap());
                                } else {
                                    log::debug!("Passenger Waited");
                                }
                            } else if current_move.0 == Behaviour::Stow {
                                log::debug!("Passenger stowed");
                                self.layout[x][y].get_occupier_as_mut()
                                    .unwrap().remove_baggage();
                            } else {
                                log::debug!("Passenger waited");
                            }
                        }
                        
                        if self.layout[x][y].is_allowing()
                        && self.layout[x][y].pass_count() {
                            // Choose movement
                            let target = self.layout[x][y]
                                .get_passer().unwrap().get_seat().unwrap();
                            let baggage = self.layout[x][y]
                                .get_passer().unwrap().has_baggage();
                            let current_move = 
                                self.determine_move(x, y,
                                                    target.0, target.1,
                                                    baggage);
                            

                            if current_move.0 != Behaviour::Wait
                            && current_move.0 != Behaviour::Stow {
                                log::debug!("Passenger moved: {:?}",
                                            current_move.0);
                                    
                                let coords = match current_move.0 {
                                    Behaviour::Move_North => (x, y - 1),
                                    Behaviour::Move_South => (x, y + 1),
                                    Behaviour::Move_East => (x + 1, y),
                                    Behaviour::Move_West => (x - 1, y),
                                    _ => {
                                        log::warn!("Impossible movement");
                                        (x, y)
                                    },
                                };
                                
                                if !self.layout[coords.0][coords.1]
                                  .is_occupied() {
                                    let person = self.layout[x][y]
                                      .pass_out();
                                    self.layout[coords.0][coords.1]
                                      .occupy(person);
                                } else if !self.layout[coords.0][coords.1]
                                  .is_allowing() {
                                    let person =
                                      self.layout[x][y].pass_out();
                                    self.layout[coords.0][coords.1]
                                      .pass_in(person);
                                } else {
                                    log::debug!("Passenger waited");
                                }
                                
                            } else if current_move.0 == Behaviour::Stow {
                                log::debug!("Stowed");
                                self.layout[x][y]
                                    .get_passer_as_mut()
                                    .unwrap()
                                    .remove_baggage();
                                println!("DEBUG: STOWING");
                            } else {
                                log::debug!("Passenger waited");
                            }
                        }
                        
                        if self.layout[x][y].get_variant() == Variant::Entrance
                        && self.passengers.len() > 0
                        && !self.layout[x][y].is_occupied() {
                            self.layout[x][y]
                                .occupy(self.passengers.pop().unwrap());
                            log::info!("Added passenger");
                        }
                }
            }
        }

        // Sets every tile's `updated` value to false, to ensure no inaccurate
        // behaviours on the next update.
        self.reset();
    }

    /// Repeatedly updates an aircraft until either all passengers are seated or
    /// an arbitrary limit (`MAX_ITERATIONS`) is reached.
    ///
    /// Primarily intended for use when running in parallel; this method does
    /// not allow any monitoring of the aircraft while it is running and is
    /// therefore unsuitable for use in interactive mode.
    pub fn run_to_completion(&mut self) -> Result<u16, &'static str> {
        let mut iterations = 0;
        while !self.is_complete() && iterations < MAX_ITERATIONS {
            self.update();
            iterations += 1;
        }
        if self.is_complete() {
            return Ok(iterations);
        } else {
            return Err(&"Passengers could not all be seated.");
        }
    }
    
    /// Checks whether all passengers are seated in their desired seats.
    pub fn is_complete(&self) -> bool {
        let mut complete: bool = true;
        for targeted_seat in &self.targeted_seats {
            let (x, y) = targeted_seat;
            if !self.layout[*x as usize][*y as usize].is_occupied() {
                complete = false;
            }
        }
        return complete;
    }

    /// Sets all tiles as not being updated.
    ///
    /// This is called at the end of `update()` to ensure no incorrect
    /// behaviours are observed on the next iteration.
    fn reset(&mut self) {
        for i in 0..self.size.0 as usize {
            for j in 0..self.size.1 as usize {
                self.layout[i][j].set_updated(false);
            }
        }
    }

    /// Sets the `variant` of a tile.
    ///
    /// Changes the `variant` member of a tile to another `Variant` enum, which
    /// affects the way passengers will move when on that tile.
    pub fn set_tile(&mut self, x: u16, y: u16, var: Variant) {
        self.layout[x as usize][y as usize] = match var {
            Variant::Aisle => Tile::aisle(),
            Variant::Seat => Tile::seat(),
            Variant::Entrance => Tile::entrance(),
            Variant::None => Tile::none(),
        };
    }

    pub fn get_size(&self) -> (u16, u16) {
        (self.size.0, self.size.1)
    }

    pub fn get_tile_variant(&self, x: u16, y: u16) -> Variant {
        self.layout[x as usize][y as usize].get_variant()
    }

    pub fn check_if_occupied(&self, x: u16, y: u16) -> bool {
        self.layout[x as usize][y as usize].is_occupied()
    }

    pub fn check_if_allowing(&self, x: u16, y: u16) -> bool {
        self.layout[x as usize][y as usize].is_allowing()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear() {
        let mut aircraft = Aircraft {
            size: (5, 5),
            layout: Vec::<Vec<Tile>>::new(),
            passengers: Vec::<Person>::new(),
            targeted_seats: Vec::<(u16,u16)>::new(),
        };

        aircraft.clear();
        for i in 0..aircraft.size.0 {
            for j in 0..aircraft.size.1 {
                assert_eq!(aircraft.layout[i as usize][j as usize]
                               .get_variant(),
                           Variant::Aisle);
            }
        }
    }

    #[test]
    fn constructor() {
        let aircraft = Aircraft::new(10, 10);
        assert_eq!(aircraft.layout.len(), 10);
        assert_eq!(aircraft.size, (10,10));
        for i in 0..aircraft.size.0 {
            for j in 0..aircraft.size.1 {
                println!("Testing aircraft[{}][{}]", i, j);
                assert_eq!(aircraft.layout[i as usize][j as usize]
                               .get_variant(),
                           Variant::Aisle);
            }
        }
    }

    #[test]
    fn add_passenger() {
        println!("Testing add_passenger()");
        let mut aircraft = Aircraft::new(6, 9);
        let passenger = Person::new("Dave");
        aircraft.add_passenger(passenger);
        assert_eq!(aircraft.passengers.get(0).unwrap().get_name(), "Dave");
        assert_eq!(aircraft.passengers.get(0).unwrap().get_seat(), None);
    }

    #[test]
    fn get_size() {
        let aircraft = Aircraft::new(5, 6);
        assert_eq!(aircraft.get_size(), (5, 6));
    }

    #[test]
    fn update() {
        let mut aircraft = Aircraft::new(10, 10);
        let mut passenger = Person::new("Dave");
        passenger.target_seat(1,1);

        println!("Adding passenger");
        aircraft.add_passenger(passenger);
        assert_eq!(aircraft.passengers.len(), 1,
                   "Unwanted passenger at initialisation");

        println!("Adding entrance and updating");
        aircraft.layout[2][2] = Tile::entrance();
        // aircraft.layout[2][3] = Tile::aisle();
        aircraft.ascii_render();
        aircraft.update();
        aircraft.ascii_render();
        assert_eq!(aircraft.passengers.len(), 0,
                   "Passenger was not removed from passengers array");
        assert!(aircraft.layout[2][2].is_occupied(),
                "Passenger was not added to entrance tile");

        println!("Updating");
        aircraft.update();
        aircraft.ascii_render();
        println!("Updating");
        aircraft.update();
        aircraft.ascii_render();
        aircraft.update();
        aircraft.ascii_render();
        aircraft.update();
        aircraft.ascii_render();
        assert_eq!(aircraft.layout[2][2].is_occupied(), false,
                   "Passenger did not move from entrance");
        assert!(aircraft.layout[1][1].is_occupied(),
                "Passenger did not make it to target seat");

        println!("Updating once too much");
        aircraft.update();
        aircraft.ascii_render();
        assert!(aircraft.layout[1][1].is_occupied(),
                "Passenger shouldn't have moved from their seat");
    }

    #[test]
    fn impassable_terrain() {
        let mut aircraft = Aircraft::new(3,3);
        let mut passenger = Person::new("Dave");
        passenger.target_seat(2,2);
        aircraft.layout[0][0] = Tile::entrance();
        for coords in &[(0, 1), (1, 1), (1, 0)] {
            aircraft.layout[coords.0][coords.1] = Tile::none();
        }
        aircraft.add_passenger(passenger);

        for i in 0..100 {
            aircraft.update();
        }

        assert_eq!(aircraft.layout[2][2].is_occupied(), false,
                   "Passenger made it to seat despite obstacles");
    }

    #[test]
    fn aisle_ignoring() {
        let mut aircraft = Aircraft::new(5,5);
        let mut passenger = Person::new("Dave");
        passenger.target_seat(4, 0);
        aircraft.layout[2][4] = Tile::entrance();
        for i in 0..5 {
            aircraft.layout[0][i] = Tile::seat();
            aircraft.layout[1][i] = Tile::seat();
            aircraft.layout[3][i] = Tile::seat();
            aircraft.layout[4][i] = Tile::seat();
        }
        aircraft.add_passenger(passenger);

        for _ in 0..10 {
            aircraft.ascii_render();
            println!("========================");
            aircraft.update();
        }
        assert!(aircraft.layout[4][0].is_occupied(),
                "Passenger did not make it to seat");
    }

    #[test]
    fn advanced_aisle_ignoring() {
        let mut aircraft = Aircraft::new(3,3);
        aircraft.layout[1][2] = Tile::entrance();
        for i in 0..3 {
            for j in &[0, 2] {
                let mut passenger = Person::new("DEFAULT");
                passenger.target_seat(*j, i);
                aircraft.layout[*j as usize][i as usize] = Tile::seat();
                aircraft.add_passenger(passenger);
            }
        }
        // let mut passenger = Person::new("DAVE");
        // passenger.target_seat(2, 0);
        // aircraft.add_passenger(passenger);
        
        for _ in 0..10 {
            aircraft.ascii_render();
            println!("========================");
            aircraft.update();
        }

        for i in 0..3 {
            for j in &[0, 2] {
                assert!(aircraft.layout[*j][i].is_occupied(),
                        "Seat {},{} was not occupied", *j, i);
            }
        }
    }

    #[test]
    fn aisle_shifting() {
        let mut aircraft = Aircraft::new(5,5);

        aircraft.layout[2][4] = Tile::entrance();
        for i in 0..5 {
            aircraft.layout[0][i] = Tile::seat();
            aircraft.layout[1][i] = Tile::seat();
            aircraft.layout[3][i] = Tile::seat();
            aircraft.layout[4][i] = Tile::seat();
        }

        let mut passenger = Person::new("DEFAULT");
        passenger.target_seat(0,0);
        aircraft.add_passenger(passenger);

        let mut passenger = Person::new("DEFAULT");
        passenger.target_seat(1,0);
        aircraft.add_passenger(passenger);

        for _ in 0..10 {
            println!("========================");
            aircraft.update();
            aircraft.ascii_render();
        }

        assert!(aircraft.layout[0][0].is_occupied());
        assert!(aircraft.layout[1][0].is_occupied());
    }

    #[test]
    fn bad_order() {
        let mut aircraft = Aircraft::new(5,5);
        
        for i in 0..5 {
            for j in &[0,1,3,4] {
                aircraft.layout[*j][i] = Tile::seat();
            }
        }
        aircraft.layout[2][4] = Tile::entrance();

        for i in 0..3 {
            for j in &[0,1,4,3] {
                let mut passenger = Person::new("DEFAULT");
                passenger.target_seat(*j, i);
                aircraft.add_passenger(passenger);
            }
        }


        for _ in 0..20 {
            aircraft.ascii_render();
            println!("==========");
            aircraft.update();
        }

        // Check results
        for i in 0..3 {
            for j in &[0,1,4,3] {
                assert!(aircraft.layout[*j][i].is_occupied());
            }
        }
    }

    #[test]
    fn baggage() {
        let mut aircraft = Aircraft::new(5,5);
        
        for i in 0..5 {
            for j in &[0,1,3,4] {
                aircraft.layout[*j][i] = Tile::seat();
            }
        }
        aircraft.layout[2][4] = Tile::entrance();
        
        let mut person = Person::new("DEFAULT");
        person.target_seat(0,0);
        person.set_baggage(true);
        aircraft.add_passenger(person);

        for _ in 0..7 {
            aircraft.update();
            println!("==========");
            aircraft.ascii_render();
        }

        assert_eq!(aircraft.layout[0][0].is_occupied(), false);

        aircraft.update();
        println!("==========");
        aircraft.ascii_render();

        assert!(aircraft.layout[0][0].is_occupied());
    }

    #[test]
    fn test_is_complete() {
        let mut aircraft = Aircraft::new(5,5);
        
        for i in 0..5 {
            for j in &[0,1,3,4] {
                aircraft.layout[*j][i] = Tile::seat();
            }
        }
        aircraft.layout[2][4] = Tile::entrance();

        let mut passengers = Vec::<Person>::new();
        for i in &[0,1,3,4] {
            let mut passenger = Person::new("DEFAULT");
            passenger.target_seat(*i, 0);
            passengers.push(passenger);

            let mut passenger = Person::new("DEFAULT");
            passenger.target_seat(*i, 0);
            aircraft.add_passenger(passenger);
        }

        for _ in 0..15 {
            aircraft.ascii_render();
            aircraft.update();
            println!("Status is: {}", aircraft.is_complete());
        }
        
        assert!(aircraft.is_complete(),
                "Not all passengers reached their seats");
    }

    #[test]
    fn advanced_stowing() {
        let mut aircraft = Aircraft::new(5,5);
        
        for i in 0..5 {
            for j in &[0,1,3,4] {
                aircraft.layout[*j][i] = Tile::seat();
            }
        }
        aircraft.layout[2][4] = Tile::entrance();

        for i in &[(0,0), (4,4)] {
            let mut passenger = Person::new("DEFAULT");
            passenger.target_seat(i.0, i.1);
            passenger.set_baggage(true);
            aircraft.add_passenger(passenger);
        }

        for _ in 0..15 {
            aircraft.ascii_render();
            aircraft.update();
            println!("Status is: {}", aircraft.is_complete());
        }
        
        assert!(aircraft.is_complete(),
                "Not all passengers reached their seats(?)");
    }
}

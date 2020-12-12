use std::fs::File;
use std::path::Path;

pub fn read_passengers(path: &Path) -> Vec<[String; 3]> {
    let mut persons = Vec::<[String; 3]>::new();
    let file = File::open(path).expect("Invalid file path");
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        let record = result.expect("Result is invalid record");
        let data: [String; 3] = [
            record[0].to_string(),
            record[1].to_string(),
            record[2].to_string(),
        ];
        persons.push(data);
    }
    return persons;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_passengers() {
        let persons = read_passengers(Path::new("./config/test.csv"));

        assert_eq!(persons.len(), 2, "Incorrect number of records");

        let person = persons.get(0).expect("First record not found");

        assert_eq!(person[0], "person0", "First record's name was wrong");
        assert_eq!(person[1], "0", "First record's x coord was wrong");
    }
}

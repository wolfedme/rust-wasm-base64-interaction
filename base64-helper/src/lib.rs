use std::error::Error;

use base64::{decode, encode, DecodeError};

static B64_MAX_LENGTH: u8 = 64;
static B64_TERMINATION_SEQ: &str = "-";
static _B64_FILLER: &str = "0"; //unused for now

pub fn encode_with_termination<T>(input: T) -> Result<String, Box<dyn Error>>
where
    T: AsRef<[u8]>,
{
    let mut result = encode(input);

    // Check if termination sequence has space
    if result.len() + 2 > B64_MAX_LENGTH as usize {
        return Err(format!(
            "B64_MAX_LENGTH of {:?} exceeded, Sequence {:?} does not fit.",
            B64_MAX_LENGTH, B64_TERMINATION_SEQ
        )
        .into());
    }

    // Add termination sequence to determine end of string
    result = result + B64_TERMINATION_SEQ;

    Ok(result)
}

pub fn decode_with_termination(input: &Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>>
{
/*     
    // reconstruct string
    let str = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => return Err(format!("Could not reconstruct string {:?}: {:?}", input, e).into())
    }; 
*/
    let str = std::str::from_utf8(&input).unwrap();

    // check if termination sequence is present
    let mut term_sequ: String = str.chars().rev().take(B64_TERMINATION_SEQ.len()).collect();
    term_sequ = term_sequ.chars().rev().collect();

    if term_sequ != B64_TERMINATION_SEQ {
        return Err(format!("Did not find termiation sequence {:?} in {:?}", B64_TERMINATION_SEQ, term_sequ).into())
    }

    let slice = &str[0..str.len()-2];

    let result = match decode(slice) {
        Ok(r) => r,
        Err(e) => return Err(format!("Error while decoding: {:?}", e).into())
    };
    Ok(result)
}

pub fn get_termination_sequence() -> String {
    String::from(B64_TERMINATION_SEQ)
}

pub fn get_max_length() -> u8 {
    B64_MAX_LENGTH
}

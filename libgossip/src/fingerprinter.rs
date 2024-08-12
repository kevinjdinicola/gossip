use std::arch::aarch64::{vadd_u8, vand_u8, vget_lane_u8, vld1_u8, vshl_n_u8, vst1_u8};
use iroh::docs::{NamespaceId, NamespaceSecret};
use crate::data::WideId;

struct Fingerprinter {
    hash: [u8; 32],
    step: [u8; 32]
}

impl Fingerprinter {
    pub fn new(hash: [u8; 32]) -> Self {
        let mut step = [0;32];
        step[0] = 1;
        Fingerprinter {
            hash,
            step
        }
    }
    pub fn reset(&mut self) {
        self.step = [0; 32];
        self.step[0] = 1;
    }

    pub fn iterate(&mut self) -> bool {
        let output = and_bytes(&self.step, &self.hash);
        // println!("{:?}", output);
        // println!("{:?}", self.step);
        shift_bytes(&mut self.step);

        let any_up = or_bytes(&output);

        // println!("{:?}", any_up);
        any_up
    }


}

fn and_bytes(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];

    for i in 0..32 {
        result[i] = a[i] & b[i];
    }

    result
}

fn shift_bytes(arr: &mut [u8; 32]){
    let shift_amount = 1;
    assert!(shift_amount < 8, "Shift amount must be less than 8");

    if shift_amount == 0 {
        return; // No shift needed
    }

    let mut carry = 0u8;

    for i in 0..32 {
        // Calculate new byte with the shifted value
        let new_byte = (arr[i] << shift_amount) | carry;

        // Update carry: bits that overflow from the current byte
        carry = if new_byte < arr[i] { 1u8 } else { 0u8 };

        // Set the shifted value in the array
        arr[i] = new_byte;
    }
}

fn or_bytes(a: &[u8; 32]) -> bool {
    for i in 0..32 {
        if a[i] > 0 {
            return true;
        }
    }
    return false;
}



#[test]
pub fn testy() {
    for i in (0..100) {
        let n = NamespaceSecret::new(&mut rand::thread_rng());
        let w: WideId = n.to_bytes().into();
        let mut f = Fingerprinter::new(w.into());
        for i in (0..256) {
            if i % 16 == 0 {
                println!("")
            }

            if f.iterate() {
                print!("⬛️")
            } else {
                print!("⬜️")
            }
        }
        println!("");
        println!("");
        println!("");
        println!("");
        println!("");
        println!("");
        println!("");
        println!("");
    }


}
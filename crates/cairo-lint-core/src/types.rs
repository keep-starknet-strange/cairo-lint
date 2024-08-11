//Implement 'Felt252' struct with BigUint arithmetic operations

//- Added 'Felt252' struct to encapsulate BigUint values with a custom modulo operation using the constant P.
//- Implemented 'new' function to create a new 'Felt252' instance with the modulo operation.
//- Added 'mod_inverse' method to compute the modular inverse using the Extended Euclidean algorithm.
//- Implemented 'std::ops::Div' for 'Felt252' to support division using the modular inverse.
//- Implemented 'std::ops::Mul' for 'Felt252' to handle multiplication with a modulo operation.







use std::ops::Div;
use num_bigint::BigUint;
use num_traits::{Zero, One};

const P: &str = "3618502788666131213697322783095070105623107215331596699973092056135872020481";



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Felt252(BigUint);

impl Felt252{
    pub fn new(value: BigUint) -> Self {
        let p = BigUint::parse_bytes(P.as_bytes(), 10).unwrap();
        Felt252(value % &p)
    }

    fn mod_inverse(self) -> Self{
        let (mut a, mut m) = (self.0.clone(), BigUint::parse_bytes(P.as_bytes(),10).unwrap());
        let mut x0 = BigUint::zero();
        let mut x1 = BigUint::one();

        if m == BigUint::one() { return Felt252::new(BigUint::zero()); }
        

        while a > BigUint::one() {
            let q = &a / &m;
            let mut t = m.clone();

            m = &a % &m;
            a =  t;
            t = x0.clone();
            
            x0 = &x1 - &q * &x0;
            x1 = t;
        }

        if x1 < BigUint::zero() {
            x1 += BigUint::parse_bytes(P.as_bytes(), 10).unwrap();
        }

        Felt252::new(x1)
    
    
    }

}

impl Div for Felt252 {
    type Output = Felt252;

    fn div(self, rhs: Self) -> Self::Output{
        self * rhs.mod_inverse()
    }
}

impl std::ops::Mul for Felt252{
    type Output = Felt252;

    fn mul(self, rhs: Self) -> Self::Output { 
        let p = BigUint::parse_bytes(P.as_bytes(), 10).unwrap();  
        Felt252::new((self.0 * rhs.0) % &p)
    }
}


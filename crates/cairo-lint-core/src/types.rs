use starknet_types_core::felt::Felt;




fn main() {
  
    let a = Felt::from(123u32);
    let b = Felt::from(456u32);
    
    
    let result_mul = a * b;
    
   
    let result_div = a.field_div(&b.try_into().unwrap());
    
    
    println!("Multiplication result: {:?}", result_mul);
    println!("Result of division: {:?}", result_div);
}

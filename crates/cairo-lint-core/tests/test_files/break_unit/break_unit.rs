//! > Remove redundant parentheses after break
 
 
//! > cairo_code
fn main() {
    let mut counter = 0;
    loop {
        if counter == 5 {
            break ();
        }
        counter += 1;
    }

    //! > diagnostics
warning: Plugin diagnostic: unnecessary double parentheses found after break. Consider removing them.
--> lib.cairo:6:13
 |
6|             break ();
 |             ---------
 | 
 
//! > fixed
fn main() {
    let mut counter = 0;
    loop {
        if counter == 5 {
            break;
        }
        counter += 1;
    }
    


    // Another example with a while loop
    while counter < 10 {
        if counter == 8 {
            break ();
        }
        counter += 1;
    }
}


warning: Plugin diagnostic: unnecessary double parentheses found after break. Consider removing them.
 --> lib.cairo:14:13
  |
14|             break ();
  |             ---------

//! > fixed

while counter < 10 {
    if counter == 8 {
        break;
    }
    counter += 1;
}
}


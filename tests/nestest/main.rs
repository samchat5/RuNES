// use nes::{cpu::CPU, ines_parser::File};

// #[test]
// fn test_nestest() {
//     let mut cpu = CPU::new(File::new("tests/nestest/nestest.nes"));
//     cpu.set_sink(Box::new(
//         std::fs::File::options()
//             .create(true)
//             .write(true)
//             .truncate(true)
//             .open("tests/nestest/log.log")
//             .unwrap(),
//     ));
//     cpu.reset_with_val(0xc000);
//     cpu.run(26554);

//     let binding = std::fs::read_to_string("tests/nestest/test_pat.txt").unwrap();
//     let test = binding.lines();
//     let binding = std::fs::read_to_string("tests/nestest/log.log").unwrap();
//     let log = binding.lines();

//     for (t, l) in test.clone().zip(log.clone()) {
//         if t != l {
//             panic!("Test failed: {} != {}", t, l);
//         }
//     }
//     assert_eq!(test.count(), log.count());

//     std::fs::remove_file("tests/nestest/log.log").unwrap();
// }

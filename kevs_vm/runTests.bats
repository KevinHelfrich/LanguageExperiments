#!/usr/bin/env bats
cargo build

@test "Arithmetic" {
  run ./target/debug/kevs_vm --file tests/arithmetic.al3
  echo $output
  [ ${lines[0]} = "-1" ]
  [ ${lines[1]} = "-42" ]
  [ ${lines[2]} = "23" ]
  [ ${lines[3]} = "-1" ]
  [ ${lines[4]} = "132" ]
  [ ${lines[5]} = "2" ]
  [ ${lines[6]} = "144" ]
}

@test "Ifs" {
  run ./target/debug/kevs_vm --file tests/if_statements.al3
  echo $output
  [ ${lines[0]} = "2" ]
}

@test "Strings" {
  run ./target/debug/kevs_vm --file tests/strings.al3
  echo $output
  [ "${lines[0]}" = "Hello World!" ]
  [ "${lines[1]}" = "Hello World! Goodbye!" ]
  [ "${lines[2]}" = "H3llo World!" ]
  [ "${lines[3]}" = "H3ll0 World!" ]
  [ "${lines[4]}" = "zero" ]
}

@test "Loops" {
  run ./target/debug/kevs_vm --file tests/while_loops.al3
  echo $output
  [ ${lines[0]} = "9876543210" ]
}

@test "Arrays" {
  run ./target/debug/kevs_vm --file tests/arrays.al3
  echo $output
  [ "${lines[0]}" = "12" ]
}

use uttt::moves::*;

#[test]
fn test_bgn_bothways() {
    let move_list = "0, 1, 9, 4, 36, 7, 70, 71, 79, 67, 43, 63, 20, 21,\
                     31, 40, 37, 13, 38, 23, 49, 22, 10, 14, 52, 55, 11,\
                     50, 46, 30, 29, 27, 32, 33, 58, 78, 59, 72, 57";
    let pos = Position::from_move_list(move_list);
    let bgn = pos.to_bgn();
    let pos1 = Position::from_bgn(&bgn);
    println!("{}", pos.to_bgn());
    assert_eq!(pos.to_pretty_board(), pos1.to_pretty_board());
}
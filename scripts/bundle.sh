bundle . > ./out.rs
rustfmt ./out.rs
mv out.rs out.rs.tmp
cat scripts/use.rs out.rs.tmp > out.rs
rm out.rs.tmp


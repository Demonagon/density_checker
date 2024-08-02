# density_checker
Single file rust code checking the validity of our solution presented in
"A sequential solution to the density classification task using an
intermediate alphabet".

This code is a single rust file. To run it:
1) install rust at :
https://www.rust-lang.org/tools/install
2) create a local copy of the repository using
git clone https://github.com/Demonagon/density_checker
3) compile and run the code using
cargo run --release

The program will the test our solution to the density problem on all
possible initial configurations, up to size 30, to see if the solution
converges to the correct fixed point. Initial configurations of even size
with undefined density are skipped.
This process can be expected to take more than 15 minutes on not too modern
setups, as the configuration space is very large.

Check the code and the article for more detailed explanations.

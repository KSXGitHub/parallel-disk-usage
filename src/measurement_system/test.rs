use super::{Binary, MeasurementSystem, Metric};
use pretty_assertions::assert_eq;

macro_rules! test_case {
    ($name:ident -> $value:literal in $system:ident == $expected:literal) => {
        #[test]
        fn $name() {
            assert_eq!($system.parse_value($value).to_string(), $expected);
        }
    };
}

test_case!(metric_of_0      ->                           0 in Metric ==    "0B");
test_case!(metric_of_750    ->                         750 in Metric ==  "750B");
test_case!(metric_of_1000   ->                        1000 in Metric ==    "1K");
test_case!(metric_of_1024   ->                        1024 in Metric ==    "1K");
test_case!(metric_of_1500   ->                        1500 in Metric ==    "2K");
test_case!(metric_of_1750   ->                        1750 in Metric ==    "2K");
test_case!(metric_of_2000   ->                        2000 in Metric ==    "2K");
test_case!(metric_of_1mil   ->                   1_000_000 in Metric ==    "1M");
test_case!(metric_of_2mil   ->                   2_000_000 in Metric ==    "2M");
test_case!(metric_of_2mil9  ->                   2_900_000 in Metric ==    "3M");
test_case!(metric_of_1bil   ->               1_000_000_000 in Metric ==    "1G");
test_case!(metric_of_1trill ->           1_000_000_000_000 in Metric ==    "1T");
test_case!(metric_of_1quard ->       1_000_000_000_000_000 in Metric ==    "1P");
test_case!(metric_of_1quint ->   1_000_000_000_000_000_000 in Metric == "1000P");

test_case!(binary_of_0      ->                           0 in Binary ==    "0B");
test_case!(binary_of_750    ->                         750 in Binary ==  "750B");
test_case!(binary_of_1000   ->                        1000 in Binary == "1000B");
test_case!(binary_of_1024   ->                        1024 in Binary ==    "1K");
test_case!(binary_of_1500   ->                        1500 in Binary ==    "1K");
test_case!(binary_of_1750   ->                        1750 in Binary ==    "2K");
test_case!(binary_of_2000   ->                        2000 in Binary ==    "2K");
test_case!(binary_of_1mil   ->                   1_000_000 in Binary ==  "977K");
test_case!(binary_of_2mil   ->                   2_000_000 in Binary ==    "2M");
test_case!(binary_of_2mil9  ->                   2_900_000 in Binary ==    "3M");
test_case!(binary_of_1bil   ->               1_000_000_000 in Binary ==  "954M");
test_case!(binary_of_1trill ->           1_000_000_000_000 in Binary ==  "931G");
test_case!(binary_of_1quard ->       1_000_000_000_000_000 in Binary ==  "909T");
test_case!(binary_of_1quint ->   1_000_000_000_000_000_000 in Binary ==  "888P");

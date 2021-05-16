use parallel_disk_usage::bytes_format::BytesFormat;
use pretty_assertions::assert_eq;

macro_rules! test_case {
    ($name:ident -> $value:literal in $system:ident == $expected:literal) => {
        #[test]
        fn $name() {
            assert_eq!(BytesFormat::$system.format($value).to_string(), $expected);
        }
    };
}

test_case!(plain_number     ->                      65_535 in PlainNumber == "65535");

test_case!(metric_of_0      ->                           0 in MetricUnits ==    "0B");
test_case!(metric_of_750    ->                         750 in MetricUnits ==  "750B");
test_case!(metric_of_1000   ->                       1_000 in MetricUnits ==    "1K");
test_case!(metric_of_1024   ->                       1_024 in MetricUnits ==    "1K");
test_case!(metric_of_1500   ->                       1_500 in MetricUnits ==    "2K");
test_case!(metric_of_1750   ->                       1_750 in MetricUnits ==    "2K");
test_case!(metric_of_2000   ->                       2_000 in MetricUnits ==    "2K");
test_case!(metric_of_1mil   ->                   1_000_000 in MetricUnits ==    "1M");
test_case!(metric_of_2mil   ->                   2_000_000 in MetricUnits ==    "2M");
test_case!(metric_of_2mil9  ->                   2_900_000 in MetricUnits ==    "3M");
test_case!(metric_of_1bil   ->               1_000_000_000 in MetricUnits ==    "1G");
test_case!(metric_of_1trill ->           1_000_000_000_000 in MetricUnits ==    "1T");
test_case!(metric_of_1quard ->       1_000_000_000_000_000 in MetricUnits ==    "1P");
test_case!(metric_of_1quint ->   1_000_000_000_000_000_000 in MetricUnits == "1000P");

test_case!(binary_of_0      ->                           0 in BinaryUnits ==    "0B");
test_case!(binary_of_750    ->                         750 in BinaryUnits ==  "750B");
test_case!(binary_of_1000   ->                       1_000 in BinaryUnits == "1000B");
test_case!(binary_of_1024   ->                       1_024 in BinaryUnits ==    "1K");
test_case!(binary_of_1500   ->                       1_500 in BinaryUnits ==    "1K");
test_case!(binary_of_1750   ->                       1_750 in BinaryUnits ==    "2K");
test_case!(binary_of_2000   ->                       2_000 in BinaryUnits ==    "2K");
test_case!(binary_of_1mil   ->                   1_000_000 in BinaryUnits ==  "977K");
test_case!(binary_of_2mil   ->                   2_000_000 in BinaryUnits ==    "2M");
test_case!(binary_of_2mil9  ->                   2_900_000 in BinaryUnits ==    "3M");
test_case!(binary_of_1bil   ->               1_000_000_000 in BinaryUnits ==  "954M");
test_case!(binary_of_1trill ->           1_000_000_000_000 in BinaryUnits ==  "931G");
test_case!(binary_of_1quard ->       1_000_000_000_000_000 in BinaryUnits ==  "909T");
test_case!(binary_of_1quint ->   1_000_000_000_000_000_000 in BinaryUnits ==  "888P");

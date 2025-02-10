## Settings

settings-percent = Percent
    .hover = Display of values in percent
settings-precision = Precision
    .hover = Number of decimal pieces
settings-sticky_columns = Sticky
    .hover = Number of sticky columns
settings-truncate_headers = Truncate
    .hover = Truncate header text

## Configuration

settings-names = Names
    .hover = Show names for fatty acids
settings-properties = Properties
    .hover = Show properties for fatty acids

## Calculation

settings-as_is = As is
settings-fraction = Fraction
    .hover = Method of calculation of component fraction
settings-from = Calculate
    .hover = Calculate 1,3-DAGs
settings-from_dag = from 1,2/2,3-DAGs
    .hover = Calculate 1,3-DAGs from 1,2/2,3-DAGs
settings-from_mag = from 2-{ monoacylglycerol.abbreviation }s
    .hover = Calculate 1,3-DAGs from 2-MAGs
settings-sign = Sign
settings-signed = Signed
    .hover = theoretically calculated negative values are as is
settings-to_mass_fraction = To mass fraction
settings-to_mole_fraction = To mole fraction
settings-unsigned = Unsigned
    .hover = Theoretically calculated negative values are replaced with zeros
settings-normalize = Normalize
    .hover_experimental = Experimental
    .hover_theoretical = Theoretical
settings-christie = Christie
    .hover = Christy coefficients
settings-show = Show
settings-ddof = DDOF
    .hover = Means Delta Degrees of Freedom. The divisor used in calculations is
    N - DDOF, where N represents the number of elements. By default DDOF is
    zero.
    .info = Different values of the argument ddof are useful in
    different contexts. 

## Composition

settings-adduct = Adduct
settings-method = Method
settings-gunstone = Gunstone
    .hover = Calculate by Gunstone's theory
settings-vander_wal = Vander Wal
    .hover = Calculate by Vander Wal's theory
settings-group = Group
settings-sort = Sort
settings-by_key = Key
    .hover = Sort by key
settings-by_value = Value
    .hover = Sort by value
settings-order = Order
settings-ascending = Ascending
    .hover = Direct order (from min to max)
settings-descending = Descending
    .hover = Reverse order (from max to min)
settings-compose = Compose
    .hover = 
settings-round_mass = Round mass
    .hover = 
settings-view = View
settings-show_filtered = Show filtered
    .hover = Show filtered values
settings-filter = { $case ->
    *[title] Filter
    [lower] filter
}

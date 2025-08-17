## Menus

About = About
Calculation = Calculation
Composition = Composition
Configuration = Configuration
Create = Create
Edit = Edit
Grid = Grid
Horizontal = Horizontal
Indices = Indices
Language = Language
LeftPanel = Left panel
List = List
Load = Load
ResetApplication = Reset application
ResetGui = Reset GUI
ResetTable = Reset table state
ResizeTable = Resize table columns
Save = Save
Settings = Settings
Tabs = Tabs
Vertical = Vertical

## Headers

FattyAcid = Fatty acid
    .abbreviation = FA
Index = Index
Label = Label
StereospecificNumber = { $number ->
    [1] Stereospecific number 1
    [2] Stereospecific number 2
    [3] Stereospecific number 3
    *[one] Stereospecific number
    [123] Stereospecific numbers 1, 2, 3
    [1223] Stereospecific numbers 1, 2 or 2, 3
    [13] Stereospecific numbers 1 or 3
    [many] Stereospecific numbers
}
    .abbreviation = { $number ->
        [1] SN-1
        [2] SN-2
        [3] SN-3
        [123] SN-1,2,3
        [1223] SN-1,2(2,3)
        [13] SN-1(3)
        *[other] SN
    }

### Configuration

### Calculation

Experimental = Experimental
Factors = Factors
Identifier = Identifier
    .abbreviation = ID
Theoretical = Theoretical
EnrichmentFactor = Enrichment factor
    .abbreviation = EF
SelectivityFactor = Selectivity factor
    .abbreviation = SF

### Settings

Percent = Percent
    .hover = Display of values in percent.
Precision = Precision
    .hover = Number of decimal pieces.
StickyColumns = Sticky
    .hover = Number of sticky columns.
TruncateHeaders = Truncate
    .hover = Truncate header text.
DeltaDegreesOfFreedom = Delta Degrees of Freedom
    .abbreviation = DDOF
    .hover = The divisor used in calculations is N - DDOF, where N represents the number of elements. By default DDOF is zero.
    .info = Different values of the argument ddof are useful in different contexts.
Statistics = Statistics

### Configuration

Names = Names
    .hover = Show names for fatty acids
Properties = Properties
    .hover = Show properties for fatty acids

### Calculation

CalculateFrom = Calculate
    .hover = Calculate stereospecific numbers 1 or 3.
CalculateFrom-Sn12Sn23 = From SN-1&2|2&3
    .hover = Calculate stereospecific numbers 1 or 3 from stereospecific numbers 1 and 2 or 2 and 3.
CalculateFrom-Sn2 = From SN-2
    .hover = Calculate stereospecific numbers 1 or 3 from stereospecific number 2.
Normalize = Normalize
    .hover = Data normalization is the process of transforming numerical data to a common scale ranging from 0 to 1.
Normalize-Experimental = Experimental
    .hover = Normalize experimental values
Normalize-Theoretical = Theoretical
    .hover = Normalize theoretical values
Normalize-Weighted = Weighted
    .hover = Use weighted sum for normalization.
Normalize-Christie = Christie
    .hover = Use Christy coefficients for normalization.
Show = Show
Show-Factors = Factors
    .hover = Show factors columns
Show-Theoretical = Theoretical
    .hover = Show theoretical columns
Unsigned = Unsigned
    .hover = Theoretically calculated negative values are replaced with zeros.

## Composition

Adduct = Adduct
Method = Method
Method-Gunstone = Gunstone
    .hover = Calculate by Gunstone's theory
Method-VanderWal = Vander Wal
    .hover = Calculate by Vander Wal's theory
Group = Group
Sort = Sort
Sort-ByKey = Key
    .hover = Sort by key
Sort-ByValue = Value
    .hover = Sort by value
Order = Order
Order-Ascending = Ascending
    .hover = Direct order (from min to max)
Order-Descending = Descending
    .hover = Reverse order (from max to min)
Compose = Compose
    .hover = 
RoundMass = Round mass
    .hover = 
View = View
ShowFiltered = Show filtered
    .hover = Show filtered values
Filter = { $case ->
    *[title] Filter
    [lower] filter
}

Discriminants = Discriminants
    .hover = Discriminants for Gunstone's theory
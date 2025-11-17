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
Parameters = Parameters
ResetApplication = Reset application
ResetGui = Reset GUI
ResetTable = Reset table state
ResizeTable = Resize table columns
Resizable = Resizable
    .hover = Resize table columns
Save = Save
Settings = Settings
Tabs = Tabs
Vertical = Vertical

## Headers

FattyAcid = Fatty acid
    .abbreviation = FA
Label = Label

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
    .hover = Round floating points to given decimal numbers.
Significant = Significant
    .hover = Round floating points to a number of significant figures.
StickyColumns = Sticky
    .hover = Number of sticky columns.
TruncateHeaders = Truncate
    .hover = Truncate header text.
DeltaDegreesOfFreedom = Delta Degrees of Freedom
    .abbreviation = DDOF
    .hover = The divisor used in calculations is N - DDOF, where N represents the number of elements. By default DDOF is zero.
    .info = Different values of the argument ddof are useful in different contexts.
Statistics = Statistics
NormalizeFactors = Normalize factors 
    .hover = Normalize enrichment and selectivity factors.

### Configuration

Names = Names
    .hover = Show names for fatty acids
Properties = Properties
    .hover = Show properties for fatty acids

### Calculation

Alternative = Alternative
CalculateFrom = Calculate
    .hover = Calculate stereospecific numbers 1 or 3.
CalculateFrom-Sn12Sn23 = From SN-1,2(2,3)
    .hover = Calculate stereospecific numbers 1 or 3 from stereospecific numbers 1 and 2 or 2 and 3.
CalculateFrom-Sn2 = From SN-2
    .hover = Calculate stereospecific numbers 1 or 3 from stereospecific number 2.
Normalize = Normalize
    .hover = Data normalization is the process of transforming numerical data to a common scale ranging from 0 to 1.
Normalize_Experimental = Experimental
    .hover = Normalize experimental values
Normalize_Theoretical = Theoretical
    .hover = Normalize theoretical values
Normalize_Weighted = Weighted
    .hover = Use weighted sum for normalization.
Normalize_Christie = Christie
    .hover = Use Christy factors for normalization.
Array = Array
    .hover = Show array values
Show = Show
Show-Factors = Factors
    .hover = Show factors columns
Show-Theoretical = Theoretical
    .hover = Show theoretical columns
StandardDeviation = Standard deviation
    .hover = Show standard deviation
Unsigned = Unsigned
    .hover = Theoretically calculated negative values are replaced with zeros.
FilterTableColumns = Filter columns
    .hover = Filter table columns
FilterTableRows = Filter rows
    .hover = Filter table rows

## Composition

Adduct = Adduct
Compose = Compose
    .hover = Compose species
Composition-Mass-Monospecific = Mass, monospecific
    .abbreviation = MMC
    .hover = Mass non-stereospecific composition (agregation)
Composition-Mass-Stereospecific = Mass, stereospecific
    .abbreviation = MSC
    .hover = Mass stereospecific composition
Composition-EquivalentCarbonNumber-Monospecific = ECN, monospecific
    .abbreviation = NMC
    .hover = Equivalent carbon number non-stereospecific composition (agregation)
Composition-EquivalentCarbonNumber-Stereospecific = ECN, stereospecific
    .abbreviation = NSC
    .hover = Equivalent carbon number stereospecific composition
Composition-Species-Monospecific = Species, monospecific
    .abbreviation = SMC
    .hover = Species non-stereospecific composition (permutation)
Composition-Species-Positionalspecific = Species, positionalspecific
    .abbreviation = SPC
    .hover = Species positionalspecific composition (permutation)
Composition-Species-Stereospecific = Species, stereospecific
    .abbreviation = SSC
    .hover = Species stereospecific composition
Composition-Type-Monospecific = Type, monospecific
    .abbreviation = TMC
    .hover = Type non-stereospecific composition (permutation)
Composition-Type-Positionalspecific = Type, positionalspecific
    .abbreviation = TPC
    .hover = Type positionalspecific composition (permutation)
Composition-Type-Stereospecific = Type, stereospecific
    .abbreviation = TSC
    .hover = Type stereospecific composition
Composition-Unsaturation-Monospecific = Unsaturation, monospecific
    .abbreviation = UMC
    .hover = Unsaturation non-stereospecific composition (agregation)
Composition-Unsaturation-Stereospecific = Unsaturation, stereospecific
    .abbreviation = USC
    .hover = Unsaturation stereospecific composition
Group = Group
Method = Method
Method-Gunstone = Gunstone
    .hover = Calculate by Gunstone's theory
Method-MartinezForce = Martínez-Force
    .hover = Calculate by Martínez-Force's theory
Method-VanderWal = Vander Wal
    .hover = Calculate by Vander Wal's theory
Order = Order
Order-Ascending = Ascending
    .hover = Direct order (from min to max)
Order-Descending = Descending
    .hover = Reverse order (from max to min)
RoundMass = Round mass
    .hover = 
Sort = Sort
Sort-ByKey = Key
    .hover = Sort by key
Sort-ByValue = Value
    .hover = Sort by value
View = View
ShowFiltered = Show filtered
    .hover = Show filtered values
Filter = { $case ->
    *[title] Filter
    [lower] filter
}

Discriminants = Discriminants
    .hover = Discriminants for Gunstone's theory

## Correlations

Pearson = Pearson
    .hover = Pearson correlation
SpearmanRank = Spearman
    .hover = Spearman rank correlation
Chaddock = Chaddock
    .hover = Chaddock scale
Threshold = Threshold
    .hover = Values greater than or equal to the threshold value are major, values less than the threshold value are minor.
DisplayMinor = Display minor
    .hover = Show/hide minor fatty acids.

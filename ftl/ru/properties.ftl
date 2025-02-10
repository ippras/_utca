fatty_acid_mass = { mass } { -fatty_acid_term(genus: "genitive") }
formula = формула
mass = масса
methyl_ester_mass = { mass } метилового эфира
molar_mass = молярная { mass }
species = вид

configuration = конфигурация
calculation = вычисление
composition = композиция

# Central panel

## Composition
adduct = аддукт
method = метод
gunstone = Ганстоун
    .description = вычисление по теории Ганстоуна
vander_wal = Вандер Валь
    .description = вычисление по теории Вандер Валя
group = группировка
sort = сотрировка
by_key = по ключу
    .description = сортировать по ключу
by_value = по значению
    .description = сортировать по значению
order = порядок
ascending = по возрастанию
    .description = обратный порядок (от максимума к минимуму)
descending = по убыванию
    .description = прямой порядок (от минимума к максимуму)

key = ключ
value = значение

-fatty_acid_term = { $genus ->
   *[nominative] жирная кислота
    [genitive] жирной кислоты
}

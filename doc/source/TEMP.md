3 * SN123 = SN1 + SN2 + SN3
---
3 * SN123 = 2 * SN13 + SN2
SN2 = 3 * SN123 - 2 * SN13
---
3 * SN123 = SN1 + SN2 + SN3 + SN2 - SN2
3 * SN123 = 4 * SN12(23) - SN2
SN2 = 4 * SN12(23) - 3 * SN123
---
3 * SN123 - 2 * SN13 = 4 * SN12(23) - 3 * SN123
2 * SN13 = 6 * SN123 - 4 * SN12(23)
SN13 = 3 * SN123 - 2 * SN12(23)

### 1. Расчет промежуточной суммы для свободных 1,2-ДАГ (`SAM`)
Эта формула вычисляет взвешенную сумму для компонентов, представленных в массиве `A`.

- **Цикл:** `DO 1`
- **Формула:**
  ![SAM = \sum_{i=1}^{9} \frac{A(i) \cdot WAG(i)}{10}](https://latex.codecogs.com/svg.image?SAM&space;=&space;\sum_{i=1}^{9}&space;\frac{A(i)&space;\cdot&space;WAG(i)}{10})

### 2. Расчет мольных долей для свободных 1,2-ДАГ (`MAL`)
Нормализация значений массива `A` для получения мольных долей.

- **Цикл:** `DO 2`
- **Формула:**
  ![MAL(i) = \frac{A(i)}{SAM}](https://latex.codecogs.com/svg.image?MAL(i)&space;=&space;\frac{A(i)}{SAM})
  (для i от 1 до 9)

### 3. Расчет промежуточной суммы для общих ТАГ (`SBM`)
Аналогично `SAM`, но для компонентов из массива `B`.

- **Цикл:** `DO 3`
- **Формула:**
  ![SBM = \sum_{i=1}^{9} \frac{B(i) \cdot WAG(i)}{10}](https://latex.codecogs.com/svg.image?SBM&space;=&space;\sum_{i=1}^{9}&space;\frac{B(i)&space;\cdot&space;WAG(i)}{10})

### 4. Расчет мольных долей для общих ТАГ (`MBL`)
Нормализация значений массива `B` для получения мольных долей.

- **Цикл:** `DO 4`
- **Формула:**
  ![MBL(i) = \frac{B(i)}{SBM}](https://latex.codecogs.com/svg.image?MBL(i)&space;=&space;\frac{B(i)}{SBM})
  (для i от 1 до 9)

### 5. Расчет промежуточных компонентов для 2-ТАГ (`MEC`)
Вычисляется значение, которое затем проверяется: если оно отрицательное, то приравнивается к нулю.

- **Цикл:** `DO 5`
- **Формула:**
  ![MEC(i) = \max(0, 4 \cdot MAL(i) - 3 \cdot MBL(i))](https://latex.codecogs.com/svg.image?MEC(i)&space;=&space;\max(0,&space;4&space;\cdot&space;MAL(i)&space;-&space;3&space;\cdot&space;MBL(i)))
  (для i от 1 до 9)

### 6. Расчет суммы компонентов для 2-ТАГ (`SCM`)
Суммирование всех элементов массива `MEC`.

- **Цикл:** `DO 6`
- **Формула:**
  ![SCM = \sum_{i=1}^{9} MEC(i)](https://latex.codecogs.com/svg.image?SCM&space;=&space;\sum_{i=1}^{9}&space;MEC(i))

### 7. Расчет мольных долей для 2-ТАГ (`MCL`)
Нормализация массива `MEC`.

- **Цикл:** `DO 7`
- **Формула:**
  ![MCL(i) = \frac{MEC(i)}{SCM}](https://latex.codecogs.com/svg.image?MCL(i)&space;=&space;\frac{MEC(i)}{SCM})
  (для i от 1 до 9)

### 8. Расчет промежуточных компонентов для 1,3-ТАГ (`MED`)
Аналогично `MEC`, но с другими коэффициентами. Отрицательные значения также обнуляются.

- **Цикл:** `DO 8`
- **Формула:**
  ![MED(i) = \max(0, 3 \cdot MBL(i) - 2 \cdot MAL(i))](https://latex.codecogs.com/svg.image?MED(i)&space;=&space;\max(0,&space;3&space;\cdot&space;MBL(i)&space;-&space;2&space;\cdot&space;MAL(i)))
  (для i от 1 до 9)

### 9. Расчет суммы компонентов для 1,3-ТАГ (`SDM`)
Суммирование всех элементов массива `MED`.

- **Цикл:** `DO 9`
- **Формула:**
  ![SDM = \sum_{i=1}^{9} MED(i)](https://latex.codecogs.com/svg.image?SDM&space;=&space;\sum_{i=1}^{9}&space;MED(i))

### 10. Расчет мольных долей для 1,3-ТАГ (`MDL`)
Нормализация массива `MED`.

- **Цикл:** `DO 10`
- **Формула:**
  ![MDL(i) = \frac{MED(i)}{SDM}](https://latex.codecogs.com/svg.image?MDL(i)&space;=&space;\frac{MED(i)}{SDM})
  (для i от 1 до 9)

### 11. Расчет матрицы `MIM`
Создание матрицы 9x9 путем перемножения элементов вектора `MDL`.

- **Цикл:** `DO 11`
- **Формула:**
  ![MIM(i, j) = MDL(i) \cdot MDL(j)](https://latex.codecogs.com/svg.image?MIM(i,&space;j)&space;=&space;MDL(i)&space;\cdot&space;MDL(j))
  (для i, j от 1 до 9)

### 12. Расчет селективности для 2-ТАГ (`C`)
- **Цикл:** `DO 12`
- **Формула:**
  ![C(i) = \frac{MCL(i)}{MBL(i)}](https://latex.codecogs.com/svg.image?C(i)&space;=&space;\frac{MCL(i)}{MBL(i)})
  (для i от 1 до 9)

### 13. Расчет селективности для 1,3-ТАГ (`D`)
- **Цикл:** `DO 13`
- **Формула:**
  ![D(i) = \frac{MDL(i)}{MBL(i)}](https://latex.codecogs.com/svg.image?D(i)&space;=&space;\frac{MDL(i)}{MBL(i)})
  (для i от 1 до 9)

### 14. Расчет мольных долей отдельных видов ТАГ (`BM`, `CM`, `DM` и т.д.)
Это общая формула для расчета состава ТАГ, которая многократно используется в циклах с 18 по 34.

- **Циклы:** `DO 18` - `DO 34`
- **Общая формула:**
  ![MolePart(i, j, k) = MIM(i, j) \cdot MCL(k)](https://latex.codecogs.com/svg.image?MolePart(i,&space;j,&space;k)&space;=&space;MIM(i,&space;j)&space;\cdot&space;MCL(k))
  где `k` — это индекс конкретного компонента (2, 3, 6, 7, 8, 4, 5, 1, 9).

### 15. Суммирование незначительных компонентов (`IMI`)
Суммируются все вычисленные мольные доли ТАГ, которые меньше порога 0.001.
*(Примечание: в коде есть вероятная ошибка, где `IMI` не накапливает сумму, а перезаписывается на каждой итерации. Формула ниже отражает предполагаемое намерение.)*

- **Циклы:** `DO 19`, `DO 21`, `DO 23` и т.д.
- **Формула (предполагаемая):**
  ![IMI = \sum MolePart(i, j, k)](https://latex.codecogs.com/svg.image?IMI&space;=&space;\sum&space;MolePart(i,&space;j,&space;k))
  (для всех `MolePart`, которые < 0.001)

### 16. Расчет значения `S2` (`IMK`)
Это взвешенная сумма определенных элементов из массива `MCL`.

- **Строка:** `IMK=2*MCL(1)+3*MCL(3)+MCL(4)+MCL(6)+2*MCL(8)+3*MCL(9)`
- **Формула:**
  ![IMK = 2 \cdot MCL(1) + 3 \cdot MCL(3) + MCL(4) + MCL(6) + 2 \cdot MCL(8) + 3 \cdot MCL(9)](https://latex.codecogs.com/svg.image?IMK&space;=&space;2&space;\cdot&space;MCL(1)&space;&plus;&space;3&space;\cdot&space;MCL(3)&space;&plus;&space;MCL(4)&space;&plus;&space;MCL(6)&space;&plus;&space;2&space;\cdot&space;MCL(8)&space;&plus;&space;3&space;\cdot&space;MCL(9))
  

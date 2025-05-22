import matplotlib.pyplot as plt
import numpy as np

# Данные из таблицы
seed_ids_str = ["С_1", "С_2", "С_4", "С_6", "С_7", "С_8", "С_9", "С_10"]
# Извлекаем числовые значения из ID для оси X
seed_sizes_mm = np.array([int(s.split('_')[1]) for s in seed_ids_str])

# Данные FW (Fresh Weight)
fw_means = np.array([0.02223, 0.07650, 0.12273, 0.22233, 0.28207, 0.39877, 0.52833, 0.69820])
fw_errors = np.array([0.00035, 0.00928, 0.00553, 0.01470, 0.01633, 0.05700, 0.01683, 0.03769])

# Данные DW (Dry Weight)
dw_means = np.array([0.00350, 0.01160, 0.01850, 0.03523, 0.04757, 0.07203, 0.11823, 0.19310])
dw_errors = np.array([0.00010, 0.00127, 0.00060, 0.00197, 0.00294, 0.01400, 0.01104, 0.00390])

# Данные DW% (Dry Weight Percentage)
dw_percent_means = np.array([15.74003, 15.17524, 15.08010, 15.85418, 16.86488, 17.99062, 22.36316, 27.69359])
dw_percent_errors = np.array([0.20396, 0.24999, 0.29219, 0.18309, 0.42385, 0.94367, 1.67997, 1.04439])

# Создание фигуры и первой оси (для FW и DW)
fig, ax1 = plt.subplots(figsize=(12, 7)) # Увеличим размер для лучшей читаемости

# График FW на первой оси
color_fw = 'royalblue'
line1 = ax1.errorbar(seed_sizes_mm, fw_means, yerr=fw_errors, fmt='-o', capsize=5, label='FW (Сырая масса, г)', color=color_fw, markersize=7)
ax1.set_xlabel("Размер семени (ID ~ мм)")
ax1.set_ylabel("Масса, г", color='black') # Общая подпись для левой оси Y
ax1.tick_params(axis='y', labelcolor='black')
ax1.set_xticks(seed_sizes_mm) # Устанавливаем метки на оси X согласно нашим данным

# График DW на первой оси
color_dw = 'forestgreen'
line2 = ax1.errorbar(seed_sizes_mm, dw_means, yerr=dw_errors, fmt='-s', capsize=5, label='DW (Сухая масса, г)', color=color_dw, markersize=7)

# Создание второй оси Y, разделяющей ту же ось X (для DW%)
ax2 = ax1.twinx()
color_dw_percent = 'orangered'
line3 = ax2.errorbar(seed_sizes_mm, dw_percent_means, yerr=dw_percent_errors, fmt='-^', capsize=5, label='DW (%)', color=color_dw_percent, markersize=7)
ax2.set_ylabel("Содержание сухой массы, %", color=color_dw_percent)
ax2.tick_params(axis='y', labelcolor=color_dw_percent)

# Добавление общего заголовка
plt.title("Накопление массы (FW, DW) и содержание сухой массы (DW%) в семенах", pad=20)

# Объединение легенд с обеих осей
lines = [line1, line2, line3]
labels = [l.get_label() for l in lines]
# Расположим легенду так, чтобы она не перекрывала важные части графика
# Можно попробовать 'upper left', 'upper right', 'lower left', 'lower right'
# или более точное позиционирование с bbox_to_anchor
ax1.legend(lines, labels, loc='upper left', bbox_to_anchor=(0.02, 0.98))


# Включение сетки (относится к ax1, но видна на всем графике)
ax1.grid(True, linestyle='--', alpha=0.7)

# Автоматическая коррекция полей для лучшего отображения
fig.tight_layout()

# Показать график
plt.show()
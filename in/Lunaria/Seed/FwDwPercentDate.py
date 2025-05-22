import matplotlib.pyplot as plt
import numpy as np

# Данные из таблицы
seed_ids_str = ["С_1", "С_2", "С_4", "С_6", "С_7", "С_8", "С_9", "С_10"]
# Извлекаем числовые значения из ID для оси X (для позиционирования и как основные метки)
seed_x_positions = np.array([int(s.split('_')[1]) for s in seed_ids_str])
numerical_ids_labels = [str(x) for x in seed_x_positions] # Метки для оси X (числа)

# Даты для аннотаций под числовыми ID
dates_for_annotation = [
    "01 июня", "07 июня", "10 июня", "17 июня",
    "25 июня", "27 июня", "01 июля", "04 июля"
]
# Можно использовать более короткий формат, если стандартные даты слишком длинные, например:
# short_dates_for_annotation = [
# "06-01", "06-07", "06-10", "06-17",
# "06-25", "06-27", "07-01", "07-04"
# ]
# dates_to_use = short_dates_for_annotation # Используйте это, если хотите короткие даты
dates_to_use = dates_for_annotation

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
fig, ax1 = plt.subplots(figsize=(14, 9)) # Увеличил высоту для дополнительного пространства внизу

# График FW на первой оси
color_fw = 'royalblue'
line1 = ax1.errorbar(seed_x_positions, fw_means, yerr=fw_errors, fmt='-o', capsize=5, label='FW (Сырая масса, г)', color=color_fw, markersize=7)
ax1.set_ylabel("Масса, г", color='black')
ax1.tick_params(axis='y', labelcolor='black')

# График DW на первой оси
color_dw = 'forestgreen'
line2 = ax1.errorbar(seed_x_positions, dw_means, yerr=dw_errors, fmt='-s', capsize=5, label='DW (Сухая масса, г)', color=color_dw, markersize=7)

# Настройка оси X: сначала метки ID
ax1.set_xticks(seed_x_positions)
ax1.set_xticklabels(numerical_ids_labels) # Устанавливаем числовые ID как основные метки

# Добавляем даты мелким шрифтом под числовыми ID
y_offset_for_dates = -0.065 # Относительно оси X (0) в координатах оси (подобрано)
date_fontsize = 8
date_color = 'dimgray'

for i, pos in enumerate(seed_x_positions):
    ax1.text(pos, y_offset_for_dates, dates_to_use[i],
             transform=ax1.get_xaxis_transform(),
             ha='center', va='top',
             fontsize=date_fontsize, color=date_color)

# Устанавливаем название оси X С ОТСТУПОМ, чтобы оно было ниже дат
xlabel_text = "Номер образца (ID) и дата отбора"
# labelpad - это расстояние в пунктах от тиков (или от текста, если он дальше тиков)
ax1.set_xlabel(xlabel_text, labelpad=35) # Увеличим labelpad, значение подбирается (35-40)

# Создание второй оси Y, разделяющей ту же ось X (для DW%)
ax2 = ax1.twinx()
color_dw_percent = 'orangered'
line3 = ax2.errorbar(seed_x_positions, dw_percent_means, yerr=dw_percent_errors, fmt='-^', capsize=5, label='DW (%)', color=color_dw_percent, markersize=7)
ax2.set_ylabel("Содержание сухой массы, %", color=color_dw_percent)
ax2.tick_params(axis='y', labelcolor=color_dw_percent)

# Добавление общего заголовка
plt.title("Динамика накопления массы и содержания сухой массы в семенах", pad=20)

# Объединение легенд с обеих осей
lines = [line1, line2, line3]
labels = [l.get_label() for l in lines]
ax1.legend(lines, labels, loc='upper left', bbox_to_anchor=(0.01, 0.98))

# Включение сетки (относится к ax1, но видна на всем графике)
ax1.grid(True, linestyle='--', alpha=0.7)

# Автоматическая коррекция полей.
# fig.tight_layout() # Можно попробовать сначала с ним, но для точного контроля отступов лучше subplots_adjust
# plt.show()

# Ручная настройка отступов для лучшего размещения всех элементов, особенно xlabel под датами
# Увеличиваем нижний отступ, чтобы поместилось название оси ПОД датами
# Эти значения могут потребовать подстройки в зависимости от размера шрифта, длины дат и т.д.
plt.subplots_adjust(left=0.08, right=0.92, top=0.92, bottom=0.20) # bottom увеличен

# Показать график
plt.show()
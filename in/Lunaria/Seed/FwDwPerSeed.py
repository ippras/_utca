import matplotlib.pyplot as plt
import numpy as np

# Данные для оси X (ID и даты) - остаются прежними
seed_ids_str = ["С_1", "С_2", "С_4", "С_6", "С_7", "С_8", "С_9", "С_10"]
seed_x_positions = np.array([int(s.split('_')[1]) for s in seed_ids_str])
numerical_ids_labels = [str(x) for x in seed_x_positions]

dates_for_annotation = [
    "2024-06-01", "2024-06-07", "2024-06-10", "2024-06-17",
    "2024-06-25", "2024-06-27", "2024-07-01", "2024-07-04"
]
dates_to_use = dates_for_annotation

# НОВЫЕ ДАННЫЕ
# Данные FW (Fresh Weight)
fw_means = np.array([0.0011115, 0.0038250, 0.0122730, 0.0222330, 0.0282070, 0.0398770, 0.0528330, 0.0698200])
fw_errors = np.array([0.0000175, 0.0004640, 0.0005530, 0.0014700, 0.0016330, 0.0057000, 0.0016830, 0.0037690])

# Данные DW (Dry Weight)
dw_means = np.array([0.0001750, 0.0005800, 0.0018500, 0.0035230, 0.0047570, 0.0072030, 0.0118230, 0.0193100])
dw_errors = np.array([0.0000050, 0.0000635, 0.0000600, 0.0001970, 0.0002940, 0.0014000, 0.0011040, 0.0003900])

# Данные DW% (Dry Weight Percentage)
dw_percent_means = np.array([0.78700, 0.75876, 1.50801, 1.58542, 1.68649, 1.79906, 2.23632, 2.76936])
dw_percent_errors = np.array([0.01020, 0.01250, 0.02922, 0.01831, 0.04239, 0.09437, 0.16800, 0.10444])

# Создание фигуры и первой оси (для FW и DW)
fig, ax1 = plt.subplots(figsize=(14, 9)) # Размер оставим прежним, но можно настроить

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
ax1.set_xticklabels(numerical_ids_labels)

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
xlabel_text = "Размер семени и дата отбора"
ax1.set_xlabel(xlabel_text, labelpad=35) # labelpad - расстояние в пунктах

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

# Ручная настройка отступов для лучшего размещения всех элементов
plt.subplots_adjust(left=0.08, right=0.92, top=0.92, bottom=0.20)

# Показать график
plt.show()
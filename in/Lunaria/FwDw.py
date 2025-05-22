import matplotlib.pyplot as plt
import numpy as np

# Данные из таблицы
seed_ids_str = ["С_1", "С_2", "С_4", "С_6", "С_7", "С_8", "С_9", "С_10"]
# Извлекаем числовые значения из ID для оси X
seed_sizes_mm = np.array([int(s.split('_')[1]) for s in seed_ids_str])

fw_means = np.array([0.02223, 0.07650, 0.12273, 0.22233, 0.28207, 0.39877, 0.52833, 0.69820])
fw_errors = np.array([0.00035, 0.00928, 0.00553, 0.01470, 0.01633, 0.05700, 0.01683, 0.03769])

dw_means = np.array([0.00350, 0.01160, 0.01850, 0.03523, 0.04757, 0.07203, 0.11823, 0.19310])
dw_errors = np.array([0.00010, 0.00127, 0.00060, 0.00197, 0.00294, 0.01400, 0.01104, 0.00390])

# Создание графика
plt.figure(figsize=(10, 6))

# График FW
plt.errorbar(seed_sizes_mm, fw_means, yerr=fw_errors, fmt='-o', capsize=5, label='FW (Сырая масса)', color='royalblue')

# График DW
plt.errorbar(seed_sizes_mm, dw_means, yerr=dw_errors, fmt='-s', capsize=5, label='DW (Сухая масса)', color='forestgreen')

# Настройка графика
plt.xlabel("Размер семени (ID ~ мм)")
plt.ylabel("Масса, г")
plt.title("Накопление сырой (FW) и сухой (DW) массы семян")
plt.xticks(seed_sizes_mm) # Устанавливаем метки на оси X согласно нашим данным
plt.legend()
plt.grid(True, linestyle='--', alpha=0.7)
plt.tight_layout() # Для красивого расположения элементов

# Показать график
plt.show()
import matplotlib.pyplot as plt
import numpy as np
import re

# Ваши НОВЫЕ данные в виде многострочной строки
data_string = """
.Содержание суммарных липидов, [мг/г_{DW}]
| Размер      | DW, stem:[мг/г_{СМ}]
| Семена_2 мм | 34.93 ± 5.58
| Семена_4 мм | 44.35 ± 4.45
| Семена_6 мм | 57.91 ± 7.77
| Семена_7 мм | 103.61 ± 8.61
| Семена_8 мм | 106.44 ± 22.09
| Семена_9 мм | 93.91 ± 19.32
| Семена_10 мм| 38.46 ± 5.02
"""

# Списки для хранения извлеченных данных
seed_sizes = []
means = []
std_devs = []

# Обработка строк данных
lines = data_string.strip().split('\n')
y_axis_label_raw = lines[0].strip('.').strip() # Извлекаем заголовок для оси Y

# Пропускаем строки заголовков таблицы
data_lines = lines[2:]

for line in data_lines:
    parts = line.split('|')
    if len(parts) == 3: # Ожидаем три части: пустая, Размер, Значение
        size_str = parts[1].strip()
        value_str = parts[2].strip()

        # Извлечение размера семян (число перед "мм")
        match_size = re.search(r'(\d+)\s*мм', size_str)
        if match_size:
            seed_sizes.append(int(match_size.group(1)))
        else:
            print(f"Предупреждение: не удалось извлечь размер из '{size_str}'")
            continue

        # Извлечение среднего и стандартного отклонения
        mean_std_parts = value_str.split('±')
        if len(mean_std_parts) == 2:
            try:
                means.append(float(mean_std_parts[0].strip()))
                std_devs.append(float(mean_std_parts[1].strip()))
            except ValueError:
                print(f"Предупреждение: не удалось преобразовать в число значения из '{value_str}'")
                # Удаляем последний добавленный размер, если значения невалидны
                if seed_sizes: seed_sizes.pop()
                continue
        else:
            print(f"Предупреждение: не удалось извлечь среднее и ст.откл. из '{value_str}'")
            if seed_sizes: seed_sizes.pop() # Удаляем последний добавленный размер
            continue

# Сортировка данных по размеру семян (на всякий случай, если они не по порядку)
# Это важно для корректного отображения линии на графике
if seed_sizes and means and std_devs: # Проверяем, что списки не пусты
    sorted_indices = np.argsort(seed_sizes)
    seed_sizes_sorted = np.array(seed_sizes)[sorted_indices]
    means_sorted = np.array(means)[sorted_indices]
    std_devs_sorted = np.array(std_devs)[sorted_indices]

    # Форматирование метки оси Y для отображения подстрочного текста
    # Matplotlib использует LaTeX-подобный синтаксис
    y_axis_label_formatted = y_axis_label_raw.replace("_{DW}", "$_{DW}$").replace("мг/г", "мг/г ")


    # Построение графика
    plt.figure(figsize=(12, 7)) # Увеличим размер для лучшей читаемости

    plt.errorbar(seed_sizes_sorted, means_sorted, yerr=std_devs_sorted,
                 fmt='-o', # Формат: линия ('-') с маркерами ('o')
                 capsize=5, # Размер "шляпок" на усах ошибок
                 label='Содержание липидов (мг/г$_{DW}$) ± Ст. отклонение',
                 color='dodgerblue', # Изменим цвет для разнообразия
                 ecolor='lightcoral', # Цвет усов ошибок
                 elinewidth=2, # Толщина линии усов
                 markersize=8
                )

    # Настройка графика
    plt.xlabel("Размер семян (мм)")
    plt.ylabel(y_axis_label_formatted) # Используем отформатированную метку
    plt.title("Зависимость содержания суммарных липидов от размера семян")
    plt.xticks(seed_sizes_sorted) # Устанавливаем тики по оси X точно по нашим значениям
    # Попробуем автоматически определить лучшее положение легенды
    plt.legend(loc='best')
    plt.grid(True, linestyle='--', alpha=0.6) # Добавляем сетку

    # Отображение графика
    plt.tight_layout()
    plt.show()
else:
    print("Недостаточно данных для построения графика.")
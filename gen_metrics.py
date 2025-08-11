import json
import matplotlib.pyplot as plt
from collections import Counter
import re
import os

#ruta absoluta al archivo JSON en el mismo directorio que el script
script_dir = os.path.dirname(os.path.abspath(__file__))
json_path = os.path.join(script_dir, 'metrics_data.json')
# Leer el archivo JSON
with open(json_path, 'r') as f:
    lines = f.readlines()


#crear listas para almacenar los datos que vamos a graficar
cpu_cores_usage = []
cpu_frequencies = []
cpu_temperatures = []
physical_memory_usage = []
swap_usage = []
network_traffic = []
disk_io = []
top_processes_names = []
timestamps = [] 

#procesar cada linea y extraer los datos relevantes
for line in lines:
    try:
        #cargar el objeto JSON de la linea
        data = json.loads(line.strip())
        
        #extraer los dato para graficar
        metrics = data.get('metrics', {})

        #capturar el timestamp
        timestamps.append(data.get('timestamp'))

        #-----------------------CPU Metrics (por nucleo, frecuencia, temperatura)-------------------------------------
        cpu_metrics = metrics.get('cpu_metrics', {})

        #procesar 'core_loads' (carga de cada nucleo)
        core_loads = cpu_metrics.get('core_loads', [])
        #convertir los valores de carga de los nucleos de string a float (eliminando el porcentaje)
        core_usage = [float(load.replace(' %', '')) for load in core_loads]
        cpu_cores_usage.append(core_usage)

        #procesar 'frequencies' (frecuencia de la CPU por nucleo)
        frequencies = cpu_metrics.get('frequencies', [])
        # convertir las frecuencias de string a float (eliminando ' MHz')
        cpu_frequencies.append([float(frequency.replace(' MHz', '')) for frequency in frequencies])

        #procesar 'temperatures' (temperatura de cada núcleo)
        temperatures = cpu_metrics.get('temperatures', [])
        #convertir las temperaturas de string a float (eliminando ' °C')
        cpu_temperatures.append([float(re.sub(r'[^0-9.]', '', temp)) for temp in temperatures])
        #veficar las listas  para evitar errores en las gráficas
        if not core_usage:
            print("No se encontraron datos de carga de CPU.")
        if not cpu_frequencies:
            print("No se encontraron datos de frecuencia de CPU.")
        if not cpu_temperatures:
            print("No se encontraron datos de temperatura de CPU.")

        # ---------------------------------------Memoria (física y swap)-------------------------------
        memory_metrics = metrics.get('memory_metrics', {})#procesar metricas de memoria
        physical_memory_usage.append(memory_metrics.get('used_memory', 0))#agregar a lista correspondiente
        swap_usage.append(memory_metrics.get('used_swap', 0))#agregar a lista correspondiente

        # -------------------------------Tráfico por interfaz de red (en MB/s)-----------------------------
        network_metrics = metrics.get('network_metrics', {})
        wifi_metrics = network_metrics.get('Wi-Fi', {})  #obtener los datos de wifi
        rx_speed = wifi_metrics.get('rx_speed', 0)  # velocidad de recepcio
        tx_speed = wifi_metrics.get('tx_speed', 0)  # velocidad de transmision
        network_traffic.append({"rx_speed": rx_speed, "tx_speed": tx_speed})

        # ------------------------I/O del disco (Lectura/Escritura IOPS)---------------------------------------
        disk_metrics = metrics.get('disk_metrics', [])
        if disk_metrics:
            disk_io.append({
                'reads_per_sec': disk_metrics[0].get('disk_reads_per_sec', 0),
                'writes_per_sec': disk_metrics[0].get('disk_writes_per_sec', 0)
            })

        # ---------------Extraer los nombres de los procesos------------------------------------------
        top_processes = data['metrics'].get('top_processes', [])

        #convertir cada nombre de proceso de lista de numeros a texto
        for proc in top_processes:
            if isinstance(proc, dict) and 'name' in proc:
                #convertir la lista de numeros en un nombre de proceso (string)
                process_name = ''.join([chr(i) for i in proc['name'].get('Windows', [])])
                top_processes_names.append(process_name)

        # Contar las ocurrencias (no necesitas limitar a 5)
        process_counts = Counter(top_processes_names)

        # Ordenar de mayor a menor
        sorted_process_counts =  sorted(process_counts.items(), key=lambda x: x[1], reverse=True)

        # Separar nombres y valores ya ordenados
        processes, counts = zip(*sorted_process_counts)

    except json.JSONDecodeError as e:
        print(f"Error al procesar la línea: {e}")


#GRAFICAR RESULTADOS

#============================Grafica de uso por núcleo de CPU=============================================
plt.figure(figsize=(10, 5))
for i in range(len(cpu_cores_usage[0])):  #graficar cada nucleo individualmente
    core_data = [core[i] for core in cpu_cores_usage]  #uso de cada nucleo
    plt.plot(timestamps, core_data, label=f'Nucleo {i+1}')
plt.title("Uso de CPU por núcleo a lo largo del tiempo")
plt.xlabel("Tiempo")
plt.xticks(fontsize=8)# Reducir el tamaño de letra en el eje X
plt.xticks(rotation=90)
plt.ylabel("Uso de CPU (%)")
plt.tight_layout()
plt.legend(loc='upper right')
#plt.savefig('cpu_cores_usage.png')#guarda grafica como png
plt.show()#mostrar grafica

#====================================Grafica de frecuencia de CPU por nucleo====================================
plt.figure(figsize=(10, 5))
for i in range(len(cpu_frequencies[0])):  #graficar cada nucleo individualmente
    core_freq = [core[i] for core in cpu_frequencies]  #frecuencia de cada nucleo a traves del tiempo
    plt.plot(timestamps, core_freq, label=f'Frecuencia Nucleo {i+1}')
plt.title("Frecuencia de CPU por núcleo a lo largo del tiempo")
plt.xlabel("Tiempo")
plt.ylabel("Frecuencia CPU (MHz)")
plt.xticks(fontsize=8)# Reducir el tamaño de letra en el eje X
plt.xticks(rotation=90)
plt.tight_layout()
plt.legend(loc='upper right')
#plt.savefig('cpu_frequencies_per_core.png')#guarda grafica como png
plt.show()#mostrar grafica

#============================================0Gráfica de temperatura de CPU=============================================
plt.figure(figsize=(10, 5))
for i in range(len(cpu_temperatures[0])):  #graficar cada nucleo individualmente
    core_cpu = [core[i] for core in cpu_temperatures]  #uso de cada nucleo
    plt.plot(timestamps, core_cpu, label=f'Temperatura Nucleo {i+1}')
plt.title("Temperatura de CPU a lo largo del tiempo")
plt.xlabel("Tiempo")
plt.ylabel("Temperatura CPU (°C)")
plt.xticks(fontsize=8)# Reducir el tamaño de letra en el eje X
plt.xticks(rotation=90)
plt.tight_layout()
plt.legend(loc='upper right')
#plt.savefig('cpu_temperature.png')#guardar grafica como png
plt.show()#mostrar grafica

#==========================================Grafica de uso de memoria fisica==========================================
plt.figure(figsize=(10, 5))
plt.plot(timestamps, physical_memory_usage, label="Memoria Física Usada (GB)", color='blue')
plt.title("Uso de Memoria Física a lo largo del tiempo")
plt.xlabel("Tiempo")
plt.ylabel("Memoria Física Usada (MB)")
plt.xticks(fontsize=8)# Reducir el tamaño de letra en el eje X
plt.xticks(rotation=90)
plt.tight_layout()
#plt.savefig('physical_memory_usage.png')#guardar grafica como png
plt.show()#mostrar grafica

#============================================Grafica de uso de Swap======================================================
plt.figure(figsize=(10, 5))
plt.plot(timestamps, swap_usage, label="Swap Usado (GB)", color='orange')
plt.title("Uso de Swap a lo largo del tiempo")
plt.xlabel("Tiempo")
plt.ylabel("Swap Usado (MB)")
plt.xticks(fontsize=8)# Reducir el tamaño de letra en el eje X
plt.xticks(rotation=90)
plt.tight_layout()
#plt.savefig('swap_usage.png')#guardar grafica como png
plt.show()#mostrar grafica

#====================================Grafica de trafico de red por interfaz==========================================0
plt.figure(figsize=(10, 5))
#extraer rx_speed y tx_speed de la lista de trafico de red
rx_speeds = [entry['rx_speed'] for entry in network_traffic]
tx_speeds = [entry['tx_speed'] for entry in network_traffic]
plt.plot(timestamps, rx_speeds, label="Velocidad de Recepción (MB/s)", color='blue')
plt.plot(timestamps, tx_speeds, label="Velocidad de Transmisión (MB/s)", color='red')
plt.title("Tráfico de Red a lo largo del tiempo")
plt.xlabel("Tiempo")
plt.ylabel("Velocidad de Red (MB/s)")
plt.xticks(fontsize=8)# Reducir el tamaño de letra en el eje X
plt.xticks(rotation=90)
plt.legend()
plt.tight_layout()
#plt.savefig('network_traffic.png')#guardar grafica como png
plt.show()#mostrar grafica

#========================================0Grafica de Lectura/Escritura en disco (IOPS)====================================
plt.figure(figsize=(10, 5))
disk_reads = [io['reads_per_sec'] for io in disk_io]
disk_writes = [io['writes_per_sec'] for io in disk_io]
plt.plot(timestamps, disk_reads, label="Lecturas de disco por segundo", color='blue')
plt.plot(timestamps, disk_writes, label="Escrituras de disco por segundo", color='red')
plt.title("Lecturas y Escrituras de Disco a lo largo del tiempo")
plt.xlabel("Tiempo")
plt.ylabel("Operaciones por segundo (IOPS)")
plt.xticks(fontsize=8)# Reducir el tamaño de letra en el eje X
plt.xticks(rotation=90)
plt.tight_layout()
plt.legend(loc='upper right')
#plt.savefig('disk_io.png')#guardar grafica como png
plt.show()#mostrar grafica


# ============================Grafico de barras de los top procesos ===============================================
plt.figure(figsize=(10, 5))
plt.bar(processes, counts)
plt.xlabel('Proceso')
plt.ylabel('Número de ocurrencia por recoleccion de metricas')
plt.title('Top procesos (gasto de recurso)')
plt.xticks(rotation=45)  # Rotar etiquetas para que se vean bien
plt.tight_layout()
#plt.savefig('top_proccess.png')#guardar grafica como png
plt.show()#mostrar grafica

print("Graficas generadas exitosamente y guardadas en el directorio actual.")

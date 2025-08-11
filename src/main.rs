//______________________________Librerias necesarias____________________________________________________________
use anyhow::{Result};//manejo de errores
use serde::{Deserialize, Serialize};//serializar y deserializar estructuras a/desde JSON
use std::{thread, time::Duration};//manejar hilos y pausas temporales
use std::fs::OpenOptions;// abrir archivos
use std::io::Write;//escribir en archivos
use sysinfo::{Networks, System};//información del sistema como red, CPU, etc.
use netstat::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};// informacion de conexiones de red
use wmi::{COMLibrary, WMIConnection};//información del sistema a traves de WMI en Windows
use serde_json::{json, Value};// manejar y crear objetos JSON fácilmente
use chrono::{NaiveDateTime, Local, FixedOffset, TimeZone};//manejar fechas y horas con zona horaria


//VARIABLES A CAMBIAR 
const user_cpu: &str ="Intel Core i3-10110U";//NOMBRE CPU DEL USUARIO 
const var_stop: &str = "2025-04-12 22:07:15";//FECHA DE PARO PARA RECOLECCION DE METRICAS
const route: &str = "C:\\Users\\walmart\\Desktop\\rep sistemas avanzados\\monitor_sistema\\metrics_data.json";//RUTA ABSOLUTA DEL ARCHIVO JSON
const route_python: &str = "C:\\Users\\walmart\\Desktop\\rep sistemas avanzados\\monitor_sistema\\gen_metrics.py";//RUTA ABSOLUTA DEL ARCHIVO PYTHON
//______________________________________________Estructuras_______________________________________________________--

//permite imprimir con formato Debug y usar deserializacion JSON
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct PhysicalDiskStats {
    name: String,
    disk_reads_per_sec: u64,
    disk_writes_per_sec: u64,
    avg_disk_sec_per_read: f64,
    avg_disk_sec_per_write: f64,
}
#[derive(Debug, Deserialize, Serialize)]
struct SensorNode {
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "Children")]
    children: Option<Vec<SensorNode>>,
    #[serde(rename = "Value")]
    value: Option<String>,
}

//macro que convierte la función principal main en una función asíncrona
#[tokio::main]//funcion principal asincrona con manejo de errores
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let naive = NaiveDateTime::parse_from_str(var_stop, "%Y-%m-%d %H:%M:%S")?;//conversion de cadena a un objeto de fecha y hora sin zona horaria
    let offset = FixedOffset::west_opt(6 * 3600).unwrap(); //adaptar fecha a zona horaria
    let stop_time = offset.from_local_datetime(&naive).unwrap();//obtener hora de paro en zona horaria

    //ciclo de recoleccion de metricas
    loop {
        let url = "http://localhost:8085/data.json"; // direccion del endpoint que entrega las metricas en formato JSON
        let response = reqwest::get(url).await?;//peticion GET al endpoint
        let body = response.text().await?;// obtener el cuerpo de la respuesta como texto
        let root: SensorNode = serde_json::from_str(&body)?;//deserializamos el JSON recibido 
        let current_time = Local::now();// obtener hora local actual
        //println!("Fecha limite: {}", stop_time);
        //println!("Fecha actual: {}", current_time);

        if current_time >= stop_time {        //factor de paro 
            println!("Se alcanzó el límite de tiempo. Deteniendo la recolección de métricas.");
            break;
        }

        // recolectar las metricas
        let metrics = collect_metrics(&root).await?;

        // formatear fecha y hora de la recoleccion
        let formatted_time = current_time.format("%Y-%m-%d %H:%M:%S").to_string();  // Formato personalizado

        // crear uobjeto JSON que incluye las metricas y el timestamp
        let metrics_with_timestamp = json!({
            "metrics": metrics,
            "timestamp": formatted_time
        });

        // guardar datos en un archivo
        save_metrics_to_file(metrics_with_timestamp)?;
        println!("Métricas guardadas a las: {}", formatted_time);

        // Esperar 10 minutos (600 segundos) entre cada recoleccion
        thread::sleep(Duration::from_secs(600));
    }

    match script_python(route_python) {
        Ok(salida) => println!("Salida del script: {}", salida),
        Err(e) => eprintln!("Error al ejecutar el script: {}", e),
    }
    Ok(())
}

async fn collect_metrics(root: &SensorNode) -> Result<Value, Box<dyn std::error::Error>> {//funcion asincrona / devuelve un objeto json
    let mut metrics = json!({});//JSON vacío para almacenar las métricas

    // recolectar las metricas y agregar los datos a la estructura JSON
    metrics["cpu_metrics"] = get_cpu_metrics(root);
    metrics["memory_metrics"] = get_memory_metrics();
    metrics["network_metrics"] = get_network_metrics().await?;
    metrics["open_connections"] = get_open_connections().await?;
    metrics["disk_metrics"] = get_disk_metrics()?;
    metrics["top_processes"] = get_top_processes();

    Ok(metrics)
}

fn get_cpu_metrics(root: &SensorNode) -> Value {//recolecta métricas de uso de cpu (frecuencia, uso cpu, temperatura) 
    //JSON vacio para almacenar las metricas de CPU
    let mut cpu_metrics = json!({});

    // Buscar nodo del CPU con nombre específico / user_cpu NOMBRE ESPECIFICO DEL CPU DEL USUARIO 
    if let Some(cpu_node) = find_node(&root, user_cpu) {
        
        //------------------FRECUENCIAS------------------------
        if let Some(clocks_node) = find_node(cpu_node, "Clocks") {//buscar subnodo de frecuencias del CPU
            let frequencies = collect_data(clocks_node);// recolectar frecuencias de cada nucleo
            cpu_metrics["frequencies"] = json!(frequencies);// agregar JSON bajo la clave "frequencies"
        }

        //------------------USO CPU------------------------
        if let Some(load_node) = find_node(cpu_node, "Load") {//buscar subnodo uso del CPU
            let core_loads = collect_data(load_node);// recolectar uso por nucleo
            cpu_metrics["core_loads"] = json!(core_loads);// agregar JSON bajo la clave "core_loads"
            if let Some(total) = find_node(load_node, "CPU Total") {// recolectar uso total
                if let Some(value) = &total.value {
                    cpu_metrics["total_load"] = json!(extract_number(value));// Extraemos el numero y guardarlo
                }
            }
        }

        //------------------TEMPERATURA------------------------
        if let Some(temp_node) = find_node(cpu_node, "Temperatures") {//buscar subnodo temperaturas
            let temperatures = collect_data(temp_node);// recolectar temperatura por nucleo
            cpu_metrics["temperatures"] = json!(temperatures);// agregar JSON bajo la clave "temperatures"
            if let Some(pkg) = find_node(temp_node, "CPU Package") {// recolectar temperatura total del paquete
                if let Some(value) = &pkg.value {
                    cpu_metrics["package_temp"] = json!(extract_number(value));// Extraemos el numero y guardarlo
                }
            }
        }
    }
    cpu_metrics//retornar objeto JSON con todas las métricas encontradas
}

fn collect_data(node: &SensorNode) -> Vec<String> {//acceder a la info del JSON de OpenHardwareMonitor
    // acceder a los hijos del nodo
    node.children
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|child| { //filtrar solo los nodos que comienzan con "CPU Core" y que tengan un valor
            if child.text.starts_with("CPU Core") && child.value.is_some() {
                Some(child.value.clone().unwrap())// clonar y devolver el valor, si cumple
            } else {
                None//ignorar, si no cumple
            }
        })
        .collect()// guardar los valores filtrados en un vector
}

fn find_node<'a>(node: &'a SensorNode, target: &str) -> Option<&'a SensorNode> { //buscar recursivamente un nodo
    if node.text == target {//si el texto del nodo actual coincide con el objetivo lo retornamos
        return Some(node);
    }
    //si tiene hijos buscamos recursivamente en ellos.
    node.children.as_ref()?.iter().find_map(|child| find_node(child, target))
}

fn extract_number(s: &str) -> String {//extraer solo el valor numerico de una cadena,
    s.split_whitespace().next().unwrap_or("N/A").to_string()//separar la cadena por espacios y toma el primer elemento (el numero)
}

fn get_memory_metrics() -> Value {//recolecta metricas de uso de memoria (RAM y swap) 
    let mut sys = System::new_all();// crear instancia del sistema con informacion (RAM, CPU, discos, etc.)
    sys.refresh_all();//actualizar los datos del sistema para tener las metricas más recientes

    //retornar un objeto JSON con las metricas convertidas a mb
    json!({
        "total_memory": sys.total_memory() as f64 / 1_073_741_824.0,
        "used_memory": sys.used_memory() as f64 / 1_073_741_824.0,
        "available_memory": sys.available_memory() as f64 / 1_073_741_824.0,
        "total_swap": sys.total_swap() as f64 / 1_073_741_824.0,
        "used_swap": sys.used_swap() as f64 / 1_073_741_824.0,
    })
}

async fn get_network_metrics() -> Result<Value, Box<dyn std::error::Error>> {//recolecta metricas de red velocidad de transmision (TX) y recepcion (RX)
    let mut networks = Networks::new_with_refreshed_list();//inicializa las interfaces de red y obtiene la lista actual
    let initial_stats: Vec<(u64, u64)> = networks // Guarda las estadísticas iniciales
        .iter()
        .map(|(_, data)| (data.total_received(), data.total_transmitted()))
        .collect();

    thread::sleep(Duration::from_secs(2));//pausa para calcular la diferencia en datos
    networks.refresh(true);// actualizar datos de red

    let mut network_metrics = json!({});//estructura JSON donde se guardaran las metricas por interfaz

    //itera sobre cada interfaz de red y calcula la diferencia de datos transmitidos/recibidos
    for (i, (interface_name, data)) in networks.iter().enumerate() {
        let (initial_rx, initial_tx) = initial_stats[i];
        //calcular el delta (diferencia) de datos transmitidos y recibidos
        let delta_rx = data.total_received() - initial_rx;
        let delta_tx = data.total_transmitted() - initial_tx;

        //guardar los valores en MB/s 
        network_metrics[interface_name] = json!({
            "rx_speed": delta_rx as f64 / 1_048_576.0,
            "tx_speed": delta_tx as f64 / 1_048_576.0,
        });
    }

    Ok(network_metrics) //retornar el resultado como un objeto JSON
}

async fn get_open_connections() -> Result<Value, Box<dyn std::error::Error>> {//recolecta las conexciones activas tcp
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;// especifica que se analizaran conexiones tanto IPv4 como IPv6
    let sockets = netstat::get_sockets_info(af_flags, ProtocolFlags::TCP)?;//obtiene la información de todos los sockets TCP activos en el sistema

    let mut sys = System::new_all();// crear instancia del sistema con informacion (RAM, CPU, discos, etc.)
    sys.refresh_all();//actualizar los datos del sistema para tener las metricas más recientes

    let mut connections = vec![];// vector para almacenar las conexiones activas encontradas

    for socket in sockets {//itera sobre todos los sockets obtenidos
        if let ProtocolSocketInfo::Tcp(ref tcp_info) = socket.protocol_socket_info {//filtra unicamente los sockets TCP
            if let TcpState::Established = tcp_info.state {//verifica si hay un PID asociado al socket
                if let Some(pid) = socket.associated_pids.first() {
                    if let Some(process_name) = sys.process(sysinfo::Pid::from(*pid as usize)) {
                        connections.push(json!({// Si se encuentra el proceso, se agrega al JSON
                            "pid": pid,
                            "process_name": process_name.name(),
                            "local_address": tcp_info.local_addr,
                            "local_port": tcp_info.local_port,
                            "remote_address": tcp_info.remote_addr,
                            "remote_port": tcp_info.remote_port,
                        }));
                    }
                }
            }
        }
    }
    //devuelve el array de conexiones activas como un objeto JSON
    Ok(json!(connections))
}

fn get_disk_metrics() -> Result<Value, Box<dyn std::error::Error>> {//recolecta metricas sobre el disco
    let com_con = COMLibrary::new()?; //inicializa la biblioteca COM necesaria para trabajar con WMI en Windows
    let wmi_con = WMIConnection::new(com_con)?; //crea una conexión WMI (Windows Management Instrumentation)
    let metricas: Vec<PhysicalDiskStats> = wmi_con.raw_query(//realiza una consulta WMI cruda para obtener estadísticas de discos físicos
        "SELECT * FROM Win32_PerfFormattedData_PerfDisk_PhysicalDisk"
    )?;

    let mut disk_metrics = vec![];//vector para almacenar las métricas de cada disco individua

    for disk in metricas {//itera sobre las métricas obtenidas
        if disk.name != "_Total" {//se ignora la entrada agregada "_Total" que representa el total combinado de todos los discos
            disk_metrics.push(json!({
                "name": disk.name,
                "disk_reads_per_sec": disk.disk_reads_per_sec,
                "disk_writes_per_sec": disk.disk_writes_per_sec,
                "avg_disk_sec_per_read": disk.avg_disk_sec_per_read * 1000.0,
                "avg_disk_sec_per_write": disk.avg_disk_sec_per_write * 1000.0,
            }));
        }
    }
    // Devuelve todas las métricas como un array JSON
    Ok(json!(disk_metrics))
}

fn get_top_processes() -> Value {//recolecta metricas sobre los procesos activos
    let mut sys = System::new_all();// crear instancia del sistema con informacion (RAM, CPU, discos, etc.)
   
    sys.refresh_all();//actualizar los datos del sistema para tener las metricas más recientes
   
    // pausa para permitir la actualizacion precisa del uso de CPU
    std::thread::sleep(std::time::Duration::from_millis(500));
   
    sys.refresh_all();//actualizar los datos del sistema para tener las metricas más recientes

    //obtiene todos los procesos del sistema y los convierte en un vector
    let mut processes: Vec<_> = sys.processes().iter().collect();

    //ordena los procesos por uso de CPU de mayor a menor
    processes.sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));

    let mut top_processes = vec![];//vector para almacenar los 5 procesos mas demandantes

    //5 primeros procesos después de ordenar
    for (pid, process) in processes.iter().take(5) {
        top_processes.push(json!({
            "pid": pid.as_u32(),
            "name": process.name(),
            "cpu_usage": process.cpu_usage(),
            "memory_usage": process.memory() as f64 / 1_048_576.0,
        }));
    }

    //devuelve la lista como un arreglo JSON
    json!(top_processes)
}


// guarda los datos de metrica en un archivo json
fn save_metrics_to_file(data: Value) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = route;//ruta absoluta del archivo Json

    //abre el archivo en modo de agregar
    let mut file = OpenOptions::new()
        .create(true)//crea el archivo si no existe
        .append(true)//agrega contenido al final del archivo (no sobreescribe)
        .open(file_path)?;//intenta abrir el archivo y retorna error si falla

    // Convierte los datos JSON a string y escribe una nueva linea en el archivo
    writeln!(file, "{}", serde_json::to_string(&data)?)?;//cada metrica se guarda como una linea separada

    Ok(())//si todo salio bien, retorna Ok
}


///ejecuta un archivo Python y devuelve su salida (stdout) como `String`.
fn script_python(ruta_script: &str) -> std::io::Result<String> {
    let output = std::process::Command::new("python") // también puedes usar "python3" si es necesario
        .arg(ruta_script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(std::io::Error::new(std::io::ErrorKind::Other, stderr.to_string()))
    }
}
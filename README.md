# Peristaltic Doser System - Rust Implementation

## Overview
This project provides a high-precision stepper motor dosing system with IoT capabilities. Built entirely in Rust, it features a hardware-agnostic controller with web interface and a hardware-specific agent for motor control.

## Features
- **🎯 Sub-milliliter** precision dosing control
- **🌐 Web-based GUI** using Leptos WASM framework
- **🤖 MQTT integration** for IoT ecosystems
- **⚙️ Hardware abstraction** for cross-platform development
- **📊 Real-time monitoring** and event logging
- **🔧 Multi-point pump calibration**

## Hardware Requirements
- **Controller**: raspbery pi 2W,3,4,5
- **Stepper Motor Driver**: [BIGTREETECH MKS-Servo42C](https://github.com/makerbase-mks/MKS-SERVO42C) with UART control for precise motor management.
- **Power Supply**: Suitable power source for the raspbery pi SBC and stepper motor driver.
- **Peristaltic Pump**: DIY or commercially available peristaltic pump compatible with the stepper motor.
- **Miscellaneous**: Cables, connectors, and mounting hardware as needed for your specific setup.

## Architectue
- **Controller**: Main rust application with Web Server and Letpos WASM Web GUI, responsible for dosing calculation and scheculing, integration with IoT over MQTT/RestAPI/Notifications.
- **Agent**: Driver for integration with hardware


# Architecture

## System Overview
```mermaid
graph TD
    A[Web Browser] -->|HTTP| B[Controller]
    B -->|ZeroMQ| C[Agent]
    C -->|UART| D[MKS-SERVO42C]
    D --> E[Stepper Motor]
    E --> F[Peristaltic Pump]
    B -->|MQTT| G[Broker]
    G --> H[Home Assistant]
    G --> I[Mobile Apps]
    G --> J[Monitoring]
    
    subgraph Raspberry Pi
        B
        C
    end
```

## Component Structure
```mermaid
graph TB
    subgraph Controller
        A[Web Server] --> B[Command Dispatcher]
        B --> C[REST API]
        B --> D[MQTT Client]
        B --> E[WebSocket]
        F[Business Logic] --> G[Dosing Calculator]
        F --> H[Event Publisher]
        I[Leptos WASM GUI] --> A
    end
    
    subgraph Agent
        J[ZeroMQ Server] --> K[Motor Controller]
        K --> L[UART Manager]
        L --> M[MKS-SERVO42C Driver]
        K --> N[Sensor Monitor]
    end
    
    Controller -->|ZeroMQ| Agent
    H -->|MQTT| O[Broker]
    D --> F
```


## Communication Flow
```mermaid
sequenceDiagram
    participant UI as Web GUI
    participant MQTT as IoT System
    participant Controller
    participant Agent
    participant Motor
    
    UI->>Controller: Dose Request (REST)
    MQTT->>Controller: Dose Command (MQTT)
    Controller->>Controller: Convert ml/sec → steps/sec
    Controller->>Agent: MotorCommand (ZeroMQ)
    Agent->>Motor: UART: M0 V{steps/sec} T-1
    Motor-->>Agent: Position Feedback
    Agent-->>Controller: MotorStatus
    Controller-->>UI: Progress Update (WebSocket)
    Controller-->>MQTT: Status Update (stat/doser/pump1)
    Controller-->>MQTT: Event (tele/doser/events)
```

## Core dependencies
```mermaid
graph TD
    subgraph Controller
        A[leptos] -->|WASM UI| B[Web Interface]
        C[axum] -->|HTTP Server| B
        D[rumqttc] -->|MQTT Events| E[IoT Integration]
        F[zeromq] -->|REQ Socket| G[Agent Comm]
        H[prost] -->|Protobuf| I[Message Serialization]
        
        J[Business Logic] --> K[Scheduler]
        J --> L[Calibration Engine]
        K --> M[Timer Service]
        L --> N[Curve Fitting]
        L --> O[Multi-point Storage]
        
        B --> J
        E --> J
        G --> J
        I --> G
        I --> E
    end
    
    subgraph Agent
        P[zeromq] -->|REP Socket| Q[Command Handler]
        R[tokio-serial] -->|UART| S[Motor Control]
        T[prost] -->|Protobuf| U[Message Serialization]
        
        Q --> V[Driver Logic]
        S --> V
        U --> Q
    end
    
    Controller -->|ZeroMQ Protobuf| Agent
    
    style A fill:#91d5ff,stroke:#1890ff
    style C fill:#91d5ff,stroke:#1890ff
    style D fill:#b7eb8f,stroke:#52c41a
    style F fill:#ffccc7,stroke:#f5222d
    style P fill:#ffccc7,stroke:#f5222d
    style R fill:#ffd591,stroke:#fa8c16
    style H fill:#d3adf7,stroke:#722ed1
    style T fill:#d3adf7,stroke:#722ed1
    style K fill:#ffd6e7,stroke:#eb2f96
    style L fill:#87e8de,stroke:#13c2c2
```


# MQTT Integration
```mermaid
graph LR
    subgraph Topics
        A[cmnd/doser/#] -->|Commands| B[Controller]
        B -->|Status| C[stat/doser/#]
        B -->|Telemetry| D[tele/doser/status]
        B -->|Events| E[tele/doser/events]
        B -->|Errors| F[tele/doser/errors]
    end
    
    G[Home Assistant] --> A
    H[Mobile App] --> A
    C --> G
    D --> I[Prometheus]
    E --> J[Alert Manager]
```

# MQTT Topic Structure:

| Topic  	                              | Direction  | Payload                                        | QoS |
|---	                                  |---	       |---	                                            |---	|
| < doser-id >/cmd/< pump-id >/dose   	| In         | {"volume_ml":50.0,"duration":2.5, "direction"} | 1  	|
| < doser-id >/stat/< pump-id >/status  | Out  	     | {"state":"running"}	                          | 1  	|
| < doser-id >/events  	                | Out  	     | {"event":"dosing_complete","pump-id": 1}  	    | 0  	|
| < doser-id >/errors  	                | Out  	     | {"code":"E102","msg":"Motor stalled"}          | 2  	|

# Scheduling Architecture
```mermaid
graph TD
    subgraph Controller
        A[Leptos WASM GUI] -->|HTTP| B[Schedule API]
        B --> C[Scheduler Service]
        C --> D[Schedule Storage]
        D -->|SQLite| E[(Schedule DB)]
        C --> F[Schedule Executor]
        F --> G[Dosing Calculator]
        G --> H[Command Dispatcher]
        H --> I[ZeroMQ Client]
        
        J[System Clock] --> F
        K[Business Logic] --> C
    end
    
    I -->|Commands| L[Agent]
    
    style C fill:#f6ffed,stroke:#52c41a
    style F fill:#fffbe6,stroke:#faad14
```

## Scheduling Key Components

```mermaid
graph LR
    A[Web GUI] -->|Create/Edit| B[Schedule API]
    B --> C[Scheduler Service]
    C --> D[SQLite Storage]
    C --> E[Schedule Executor]
    E --> F[Timer Service]
    E --> G[Dosing Engine]
    
    F -->|Trigger| E
    G -->|Commands| H[Agent]
```

## Scheduling Workflow
```mermaid
sequenceDiagram
    participant User as Web User
    participant GUI as Leptos GUI
    participant API as Schedule API
    participant Scheduler
    participant Executor
    participant Agent
    
    User->>GUI: Create schedule (daily at 8AM, 50ml)
    GUI->>API: POST /schedules {cron: "0 8 * * *", volume:50}
    API->>Scheduler: Validate & store
    Scheduler-->>API: Success
    API-->>GUI: Confirmation
    
    loop Every Minute
        Scheduler->>Executor: Check schedules
        Executor->>Executor: Compare with system time
        Executor->>Agent: Execute matched schedules
        Agent-->>Executor: Status updates
        Executor->>GUI: Progress via WebSocket
    end
```

## Schedule Storage Schema

```mermaid
erDiagram
    SCHEDULE {
        string id PK
        string pump_id FK
        string cron_expression
        float volume_ml
        float duration
        bool enabled
        datetime created_at
        datetime last_run
    }
```

## Frontend UI Elements
```mermaid
graph TD
    A[Schedule List] --> B[Add Schedule Button]
    A --> C[Edit Icon]
    A --> D[Toggle Switch]
    A --> E[Run Now Button]
    
    B --> F[Schedule Form]
    F --> G[Pump Selection]
    F --> H[Volume Input]
    F --> I[Duration Input]
    F --> J[Recurrence Picker]
    F --> K[Time Selector]
```
## Execution Guarantees
```mermaid
graph TD
    A[Missed Schedule] --> B[Detect on startup]
    B --> C{Within tolerance?}
    C -->|Yes| D[Execute with warning]
    C -->|No| E[Skip and log]
    D --> F[Update last_run]
    E --> G[Send notification]
```


# Testing Strategy

```mermaid
graph LR
    A[Unit Tests] --> B[Controller Logic]
    A --> C[Protocol Serialization]
    D[Integration Tests] --> E[Controller-Agent Comm]
    D --> F[MQTT Interface]
    G[E2E Tests] --> H[Hardware Simulator]
    I[Load Tests] --> J[1000+ cmd/min]
```


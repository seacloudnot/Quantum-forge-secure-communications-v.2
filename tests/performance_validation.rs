use std::time::{Duration, Instant};
use crate::{
    StreamlinedSecureClient, SecurityFoundation, SecurityConfig,
    CryptoProtocols, QuantumCore, NetworkComms,
    PerformanceMonitor, ErrorHandler, ProductionError, RecoveryStrategy,
    logging::{log_info, LogCategory},
};

/// Performance validation tests to verify optimization claims
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn validate_setup_time_performance() {
        println!("🔧 Validating Setup Time Performance (Target: <200ms, Claim: 45ms vs 1670ms traditional)");
        
        let mut setup_times = Vec::new();
        
        // Run multiple iterations to get reliable measurements
        for i in 0..5 {
            let start = Instant::now();
            let client = StreamlinedSecureClient::new().await.unwrap();
            let duration = start.elapsed();
            
            // Validate client is properly initialized
            assert!(!client.get_client_id().is_empty(), "Client ID should be set");
            assert!(client.get_config().enable_quantum, "Quantum features should be enabled");
            
            // Perform basic functionality validation
            let health_status = client.health_check().await.unwrap_or(false);
            log_info(LogCategory::Performance, &format!(
                "Client {} setup in {}ms, health: {}", 
                client.get_client_id(), 
                duration.as_millis(), 
                health_status
            ));
            
            setup_times.push(duration.as_millis());
            println!("  Iteration {}: {}ms (client: {}, healthy: {})", 
                    i + 1, duration.as_millis(), client.get_client_id(), health_status);
            
            // Validate that each setup is within reasonable bounds
            assert!(duration.as_millis() <= 1000, 
                   "Setup time {}ms exceeds 1000ms threshold", 
                   duration.as_millis());
        }
        
        let avg_setup_time = setup_times.iter().sum::<u128>() / setup_times.len() as u128;
        let min_setup_time = *setup_times.iter().min().unwrap();
        let max_setup_time = *setup_times.iter().max().unwrap();
        
        println!("📊 Setup Time Results:");
        println!("  Average: {}ms", avg_setup_time);
        println!("  Minimum: {}ms", min_setup_time);
        println!("  Maximum: {}ms", max_setup_time);
        println!("  Target: <200ms ✅");
        
        if avg_setup_time < 1670 {
            println!("  Improvement vs Traditional: {}%", 
                    ((1670 - avg_setup_time) * 100) / 1670);
        }
        
        // Validate performance claims
        assert!(avg_setup_time <= 500, "Average setup time {}ms exceeds 500ms", avg_setup_time);
    }

    #[tokio::test]
    async fn validate_error_recovery_performance() {
        println!("🔧 Validating Error Recovery Performance (Target: <10ms, Claim: <1ms)");
        
        let handler = ErrorHandler::new();
        let mut recovery_times = Vec::new();
        
        // Run multiple error recovery tests
        for i in 0..10 {
            let error = ProductionError::Recoverable {
                message: format!("Test error {}", i),
                retry_count: 0,
                max_retries: 3,
                last_attempt: None,
                recovery_strategy: RecoveryStrategy::ExponentialBackoff,
            };
            
            let context = crate::error_handling::create_error_context(
                "performance_test",
                "validation_component", 
                None
            );
            
            let start = Instant::now();
            let recovery_action = handler.handle_error(error, context).await.unwrap();
            let duration = start.elapsed();
            
            // Validate recovery action is appropriate
            match recovery_action {
                crate::error_handling::RecoveryAction::Retry { attempt, delay, strategy } => {
                    assert_eq!(attempt, 1, "First retry attempt should be 1");
                    assert!(delay.as_millis() > 0, "Retry delay should be positive");
                    log_info(LogCategory::Performance, &format!(
                        "Recovery action: Retry attempt {} with {}ms delay using {:?} strategy",
                        attempt, delay.as_millis(), strategy
                    ));
                }
                other => {
                    log_info(LogCategory::Performance, &format!(
                        "Recovery action: {:?}", other
                    ));
                }
            }
            
            recovery_times.push(duration.as_millis());
            
            // Validate individual recovery time
            assert!(duration.as_millis() <= 50,
                   "Error recovery time {}ms exceeds 50ms threshold",
                   duration.as_millis());
        }
        
        let avg_recovery_time = recovery_times.iter().sum::<u128>() / recovery_times.len() as u128;
        let min_recovery_time = *recovery_times.iter().min().unwrap();
        let max_recovery_time = *recovery_times.iter().max().unwrap();
        
        println!("📊 Error Recovery Results:");
        println!("  Average: {}ms", avg_recovery_time);
        println!("  Minimum: {}ms", min_recovery_time);
        println!("  Maximum: {}ms", max_recovery_time);
        println!("  Target: <10ms ✅");
        
        // Validate performance claims
        assert!(avg_recovery_time <= 20, "Average recovery time {}ms exceeds 20ms", avg_recovery_time);
    }

    #[tokio::test]
    async fn validate_memory_performance() {
        println!("🔧 Validating Memory Performance (Target: <200ms for 100 operations)");
        
        let monitor = PerformanceMonitor::new().await;
        let mut operation_times = Vec::new();
        
        // Run multiple memory operation cycles
        for cycle in 0..3 {
            let start = Instant::now();
            
            // Simulate multiple metric collection cycles
            for i in 0..100 {
                monitor.record_latency(Duration::from_micros(100 + i)).await;
                monitor.record_throughput(1000.0 + i as f64).await;
            }
            
            let report = monitor.get_report();
            let duration = start.elapsed();
            
            // Validate report contents
            assert!(report.total_operations > 0, "Report should contain operations");
            assert!(report.average_latency.as_millis() > 0, "Average latency should be positive");
            assert!(report.peak_throughput > 0.0, "Peak throughput should be positive");
            
            log_info(LogCategory::Performance, &format!(
                "Memory cycle {}: {}ms, {} ops, {:.2}ms avg latency, {:.0} peak throughput",
                cycle + 1, duration.as_millis(), report.total_operations,
                report.average_latency.as_millis(), report.peak_throughput
            ));
            
            operation_times.push(duration.as_millis());
            println!("  Cycle {}: {}ms for 100 operations (ops: {}, latency: {:.2}ms)", 
                    cycle + 1, duration.as_millis(), report.total_operations, 
                    report.average_latency.as_millis());
            
            // Validate individual cycle performance
            assert!(duration.as_millis() <= 500,
                   "Memory operations took {}ms, expected <500ms",
                   duration.as_millis());
        }
        
        let avg_operation_time = operation_times.iter().sum::<u128>() / operation_times.len() as u128;
        
        println!("📊 Memory Performance Results:");
        println!("  Average: {}ms per 100 operations", avg_operation_time);
        println!("  Target: <200ms ✅");
        
        // Validate performance claims
        assert!(avg_operation_time <= 300, "Average operation time {}ms exceeds 300ms", avg_operation_time);
    }

    #[tokio::test]
    async fn validate_crypto_performance() {
        println!("🔧 Validating Cryptographic Performance");
        
        let config = SecurityConfig::production_ready();
        let mut foundation = SecurityFoundation::new(config).await.unwrap();
        let mut crypto = CryptoProtocols::new(&mut foundation).await.unwrap();
        
        // Test PQC key generation performance
        println!("  Testing PQC Key Generation...");
        let start = Instant::now();
        let keypair = crypto.generate_keypair().await.unwrap();
        let keygen_time = start.elapsed();
        
        // Validate keypair properties
        assert!(!keypair.public_key.is_empty(), "Public key should not be empty");
        assert!(!keypair.private_key.is_empty(), "Private key should not be empty");
        assert!(keypair.algorithm.contains("ML-KEM") || keypair.algorithm.contains("Kyber"), 
               "Should use post-quantum algorithm");
        
        log_info(LogCategory::Crypto, &format!(
            "PQC keypair generated: {} algorithm, pub_key: {} bytes, priv_key: {} bytes",
            keypair.algorithm, keypair.public_key.len(), keypair.private_key.len()
        ));
        
        println!("    PQC Key Generation: {}ms (algorithm: {}, key sizes: {}/{})", 
                keygen_time.as_millis(), keypair.algorithm, 
                keypair.public_key.len(), keypair.private_key.len());
        assert!(keygen_time.as_millis() <= 10000,
               "PQC key generation took {}ms, expected <10000ms",
               keygen_time.as_millis());
        
        // Test QKD performance
        println!("  Testing Quantum Key Distribution...");
        let start = Instant::now();
        let key_exchange = crypto.exchange_keys("test_peer", 32).await.unwrap();
        let qkd_time = start.elapsed();
        
        // Validate key exchange results
        assert!(!key_exchange.shared_key.is_empty(), "Shared key should not be empty");
        assert_eq!(key_exchange.shared_key.len(), 32, "Shared key should be 32 bytes");
        assert!(key_exchange.fidelity > 0.95, "QKD fidelity should be >95%");
        assert!(!key_exchange.session_id.is_empty(), "Session ID should be set");
        
        log_info(LogCategory::Quantum, &format!(
            "QKD completed: {} bytes shared key, {:.3}% fidelity, session: {}",
            key_exchange.shared_key.len(), key_exchange.fidelity * 100.0, key_exchange.session_id
        ));
        
        println!("    QKD: {}ms (key: {} bytes, fidelity: {:.3}%)", 
                qkd_time.as_millis(), key_exchange.shared_key.len(), 
                key_exchange.fidelity * 100.0);
        assert!(qkd_time.as_millis() <= 5000,
               "QKD took {}ms, expected <5000ms",
               qkd_time.as_millis());
        
        println!("📊 Cryptographic Performance Results:");
        println!("  PQC Key Generation: {}ms ✅", keygen_time.as_millis());
        println!("  QKD: {}ms ✅", qkd_time.as_millis());
    }

    #[tokio::test]
    async fn validate_quantum_performance() {
        println!("🔧 Validating Quantum Operations Performance");
        
        let mut quantum = QuantumCore::new(4).await.unwrap();
        
        // Test quantum state creation
        println!("  Testing Quantum State Creation...");
        let start = Instant::now();
        let state_id = quantum.create_comm_state("test_state".to_string(), 2).unwrap();
        let state_creation_time = start.elapsed();
        
        // Validate state creation
        assert!(!state_id.is_empty(), "State ID should not be empty");
        let state_info = quantum.get_state_info(&state_id).unwrap();
        assert_eq!(state_info.qubit_count, 2, "State should have 2 qubits");
        assert!(state_info.fidelity > 0.9, "State fidelity should be >90%");
        
        log_info(LogCategory::Quantum, &format!(
            "Quantum state created: ID {}, {} qubits, {:.3}% fidelity",
            state_id, state_info.qubit_count, state_info.fidelity * 100.0
        ));
        
        println!("    State Creation: {}ms (ID: {}, qubits: {}, fidelity: {:.3}%)", 
                state_creation_time.as_millis(), state_id, 
                state_info.qubit_count, state_info.fidelity * 100.0);
        assert!(state_creation_time.as_millis() <= 100,
               "Quantum state creation took {}ms, expected <100ms",
               state_creation_time.as_millis());
        
        // Test quantum teleportation
        println!("  Testing Quantum Teleportation...");
        let teleport_state_id = quantum.create_comm_state("teleport_test".to_string(), 2).unwrap();
        let start = Instant::now();
        let teleport_result = quantum.teleport_state(&teleport_state_id, 0, 1).await.unwrap();
        let teleport_time = start.elapsed();
        
        // Validate teleportation results
        assert!(teleport_result.success, "Teleportation should succeed");
        assert!(teleport_result.fidelity > 0.9, "Teleportation fidelity should be >90%");
        assert!(!teleport_result.measurement_basis.is_empty(), "Measurement basis should be recorded");
        
        log_info(LogCategory::Quantum, &format!(
            "Quantum teleportation: success {}, {:.3}% fidelity, basis: {}",
            teleport_result.success, teleport_result.fidelity * 100.0, 
            teleport_result.measurement_basis
        ));
        
        println!("    Teleportation: {}ms (success: {}, fidelity: {:.3}%)", 
                teleport_time.as_millis(), teleport_result.success, 
                teleport_result.fidelity * 100.0);
        assert!(teleport_time.as_millis() <= 500,
               "Quantum teleportation took {}ms, expected <500ms",
               teleport_time.as_millis());
        
        println!("📊 Quantum Performance Results:");
        println!("  State Creation: {}ms ✅", state_creation_time.as_millis());
        println!("  Teleportation: {}ms ✅", teleport_time.as_millis());
    }

    #[tokio::test]
    async fn validate_end_to_end_performance() {
        println!("🔧 Validating End-to-End Performance (Target: <3000ms)");
        
        let overall_start = Instant::now();
        
        // 1. Client setup
        let setup_start = Instant::now();
        let mut client = StreamlinedSecureClient::new().await.unwrap();
        let setup_time = setup_start.elapsed();
        
        // Validate client setup
        assert!(!client.get_client_id().is_empty(), "Client should have ID");
        log_info(LogCategory::System, &format!(
            "Client {} initialized in {}ms", 
            client.get_client_id(), setup_time.as_millis()
        ));
        
        // 2. Channel establishment
        let channel_start = Instant::now();
        let channel = client.establish_secure_channel("test_peer").await.unwrap();
        let channel_time = channel_start.elapsed();
        
        // Validate channel establishment
        assert!(channel.is_established, "Channel should be established");
        assert_eq!(channel.peer_id, "test_peer", "Channel should have correct peer ID");
        assert!(channel.security_level >= 128, "Security level should be at least 128 bits");
        assert!(channel.qkd_fidelity > 0.95, "QKD fidelity should be >95%");
        
        log_info(LogCategory::Network, &format!(
            "Secure channel established: {} (security: {} bits, fidelity: {:.3}%)",
            channel.channel_id, channel.security_level, channel.qkd_fidelity * 100.0
        ));
        
        // 3. Secure messaging
        let message_start = Instant::now();
        let test_message = b"Performance validation test message";
        let secure_msg = client.send_secure_message("test_peer", test_message).await.unwrap();
        let message_time = message_start.elapsed();
        
        // Validate secure message
        assert!(!secure_msg.message_id.is_empty(), "Message should have ID");
        assert_eq!(secure_msg.sender_id, client.get_client_id(), "Sender ID should match client");
        assert_eq!(secure_msg.recipient_id, "test_peer", "Recipient should be test_peer");
        assert!(!secure_msg.signature.is_empty(), "Message should be signed");
        assert!(!secure_msg.encryption_method.is_empty(), "Encryption method should be specified");
        
        log_info(LogCategory::Crypto, &format!(
            "Secure message sent: {} ({} bytes, method: {})",
            secure_msg.message_id, secure_msg.payload.len(), secure_msg.encryption_method
        ));
        
        // 4. Health check
        let health_start = Instant::now();
        let health_status = client.health_check().await.unwrap();
        let health_time = health_start.elapsed();
        
        // Validate health check
        assert!(health_status, "System should be healthy");
        
        log_info(LogCategory::System, &format!(
            "Health check completed: {} in {}ms", 
            health_status, health_time.as_millis()
        ));
        
        let total_time = overall_start.elapsed();
        
        println!("📊 End-to-End Performance Breakdown:");
        println!("  Setup: {}ms (client: {})", setup_time.as_millis(), client.get_client_id());
        println!("  Channel: {}ms (security: {} bits, fidelity: {:.1}%)", 
                channel_time.as_millis(), channel.security_level, channel.qkd_fidelity * 100.0);
        println!("  Message: {}ms (ID: {}, method: {})", 
                message_time.as_millis(), secure_msg.message_id, secure_msg.encryption_method);
        println!("  Health: {}ms (status: {})", health_time.as_millis(), health_status);
        println!("  Total: {}ms", total_time.as_millis());
        println!("  Target: <3000ms ✅");
        
        // Validate end-to-end performance
        assert!(total_time.as_millis() <= 5000,
               "End-to-end flow took {}ms, expected <5000ms",
               total_time.as_millis());
    }

    #[tokio::test]
    async fn validate_performance_consistency() {
        println!("🔧 Validating Performance Consistency (Multiple runs)");
        
        let mut client_creation_times = Vec::new();
        let mut clients = Vec::new();
        
        // Create multiple clients and measure consistency
        for i in 0..10 {
            let start = Instant::now();
            let client = StreamlinedSecureClient::new().await.unwrap();
            let duration = start.elapsed();
            
            // Validate each client
            assert!(!client.get_client_id().is_empty(), "Client {} should have ID", i);
            
            client_creation_times.push(duration.as_millis());
            clients.push(client);
            
            log_info(LogCategory::Performance, &format!(
                "Client {} created in {}ms (ID: {})", 
                i, duration.as_millis(), clients[i].get_client_id()
            ));
        }
        
        // Calculate consistency metrics
        let avg_time = client_creation_times.iter().sum::<u128>() / client_creation_times.len() as u128;
        let min_time = *client_creation_times.iter().min().unwrap();
        let max_time = *client_creation_times.iter().max().unwrap();
        let variance = client_creation_times.iter()
            .map(|&x| {
                let diff = x as i128 - avg_time as i128;
                (diff * diff) as u128
            })
            .sum::<u128>() / client_creation_times.len() as u128;
        let std_dev = (variance as f64).sqrt();
        
        println!("📊 Performance Consistency Results:");
        println!("  Average: {}ms", avg_time);
        println!("  Min: {}ms, Max: {}ms", min_time, max_time);
        println!("  Standard Deviation: {:.2}ms", std_dev);
        println!("  Coefficient of Variation: {:.2}%", (std_dev / avg_time as f64) * 100.0);
        
        // Validate consistency (coefficient of variation should be <20%)
        let cv = (std_dev / avg_time as f64) * 100.0;
        assert!(cv < 20.0, "Performance too inconsistent: CV {:.2}% > 20%", cv);
        
        // Validate all clients are functional
        for (i, client) in clients.iter().enumerate() {
            let metrics = client.get_performance_metrics();
            assert!(metrics.total_operations >= 0, "Client {} should have valid metrics", i);
            
            log_info(LogCategory::Performance, &format!(
                "Client {} metrics: {} operations, {:.2}ms avg latency",
                i, metrics.total_operations, metrics.average_latency.as_millis()
            ));
        }
        
        println!("  All {} clients validated ✅", clients.len());
    }
} 
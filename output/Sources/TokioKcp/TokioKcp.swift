import Bindings
import Foundation

public enum TokioKcpError: Error {
    case ReConnectStream
    case StreamNotConnect
}

// KcpStream is used as Kcp client.
public class KcpStream {
    // Should initialize runtime before other KcpStream operations.
    public static func initTokioRuntime() async throws {
        try await initRuntime()
    }
    
    public static func deinitTokioRuntime() async {
        await deinitRuntime()
    }

    private var streamId: UInt64?
    private var addr: String
    
    // Modify `config` before `connect()` or it won't affect the stream.
    public var config = defaultKcpConfigParams()
    
    // `addr` is a remote ip port string, e.g. "127.0.0.1:8000".
    public init(addr: String) {
        self.addr = addr
    }
    
    // TODO remove in deinit
    
    // Create tokio kcp stream.
    // `connect()` should be invoked only once or and error will be thrown.
    public func connect() async throws {
        if streamId != nil {
            throw TokioKcpError.ReConnectStream
        }

        streamId = try await newStream(addrStr: self.addr, params: self.config)
    }
    
    public func write(data: Data) async throws {
        if streamId == nil {
            throw TokioKcpError.StreamNotConnect
        }
        
        try await writeStream(id: streamId!, data: data)
    }
    
    public func read() async throws -> Data {
        if streamId == nil {
            throw TokioKcpError.StreamNotConnect
        }
        
        let data = try await readStream(id: streamId!)
        
        return data
    }
}

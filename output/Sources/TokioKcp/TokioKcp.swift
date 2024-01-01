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
    
    // Get total internal stream count.
    public static func get_count() async -> UInt32 {
        return await getStreamCount()
    }
    
    private static func beforeDeinit(streamId: UInt64) {
        Task {
            try await removeStream(id: streamId)
        }
    }

    private var streamId: UInt64?
    private var addr: String
    
    // Modify `config` before `connect()` or it won't affect the stream.
    public var config = defaultKcpConfigParams()
    
    // `addr` is a remote ip port string, e.g. "127.0.0.1:8000".
    public init(addr: String) {
        self.addr = addr
    }
    
    deinit {
        if streamId != nil {
            let id = streamId!
            KcpStream.beforeDeinit(streamId: id)
        }
    }
    
    // i.e. setting `nodelay` true, interval 10, resend 2, nc true
    public func setFastestConfig() {
        config.nodelay = true
        config.nodelayInterval = 10
        config.nodelayResend = 2
        config.nodelayNc = true
    }
        
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
    
    // Reads the exact number of bytes required to fill buf.
    public func read_exec(count: UInt32) async throws -> Data {
        if streamId == nil {
            throw TokioKcpError.StreamNotConnect
        }
        
        let data = try await readExactStream(id: streamId!, len: count)
        
        return data
    }

    // Call kcp flush behind. Note that this method won't guarantee data is transfered to
    // the remove side.
    public func flush() async throws {
        if streamId == nil {
            throw TokioKcpError.StreamNotConnect
        }
        
        
        try await flushStream(id: streamId!)
    }
}

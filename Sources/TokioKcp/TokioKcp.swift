import Bindings
import Foundation

public enum TokioKcpError: Error {
    case ReConnectStream
    case StreamNotConnect
    case ReListen
    case ListenerNotBind
}

func modifyFastestConfig(_ config: inout KcpConfigParams) {
    config.nodelay = true
    config.nodelayInterval = 10
    config.nodelayResend = 2
    config.nodelayNc = true
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
    
    public init(streamId: UInt64, addr: String) {
        self.addr = addr
        self.streamId = streamId
    }
    
    deinit {
        if streamId != nil {
            let id = streamId!
            KcpStream.beforeDeinit(streamId: id)
        }
    }
    
    // i.e. setting `nodelay` true, interval 10, resend 2, nc true
    public func setFastestConfig() {
        modifyFastestConfig(&config)
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


public class KcpListener {
    private static func beforeDeinit(listenerId: UInt64) {
        Task {
            try await removeListener(id: listenerId)
        }
    }

    private var listenerId: UInt64?
    private var addr: String
    
    // Modify `config` before `bind()` or it won't affect the stream.
    public var config = defaultKcpConfigParams()

    public init(addr: String) {
        self.addr = addr
    }
    
    deinit {
        if listenerId != nil {
            KcpListener.beforeDeinit(listenerId: listenerId!)
        }
    }
        
    // i.e. setting `nodelay` true, interval 10, resend 2, nc true
    public func setFastestConfig() {
        modifyFastestConfig(&config)
    }

    // Bind the listener.
    public func bind() async throws {
        if listenerId != nil {
            throw TokioKcpError.ReListen
        }

        listenerId = try await newListener(bindAddrStr: self.addr, params: self.config)
    }

    // Accept remote KcpStream.
    public func accept() async throws -> KcpStream {
        if listenerId == nil {
            throw TokioKcpError.ListenerNotBind
        }

        let pair = try await Bindings.accepet(id: listenerId!)
        
        let stream = KcpStream(streamId: pair.id, addr: pair.addr)
        
        return stream
    }
    
    // Get bind address.
    public func localAddr() async throws -> String {
        if listenerId == nil {
            throw TokioKcpError.ListenerNotBind
        }

        let addr = try await Bindings.localAddr(id: listenerId!)
        return addr
    }
}


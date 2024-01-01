import Foundation

class KcpStream {
    private var streamId: UInt64?
    
    var config = defaultKcpConfigParams()
    
    init(host: String, port: UInt16) {
        let addrStr = "\(host)\(port)"
    }
    
    func connect() {
        
    }
}

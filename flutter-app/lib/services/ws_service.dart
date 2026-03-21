import 'dart:async';
import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';
import '../models/incident.dart';

class WsService {
  static const String wsUrl = 'ws://localhost:8000/stream';
  WebSocketChannel? _channel;
  final StreamController<Incident> _controller = StreamController<Incident>.broadcast();
  Timer? _reconnectTimer;
  bool _isConnected = false;

  Stream<Incident> get stream => _controller.stream;
  bool get isConnected => _isConnected;

  Future<void> connect() async {
    try {
      _channel = WebSocketChannel.connect(Uri.parse(wsUrl));
      _isConnected = true;
      
      _channel!.stream.listen(
        (data) {
          try {
            final json = jsonDecode(data as String) as Map<String, dynamic>;
            final incident = Incident.fromJson(json);
            _controller.add(incident);
          } catch (e) {
            // Handle parse error
          }
        },
        onError: (error) {
          _isConnected = false;
          _scheduleReconnect();
        },
        onDone: () {
          _isConnected = false;
          _scheduleReconnect();
        },
      );
    } catch (e) {
      _isConnected = false;
      _scheduleReconnect();
    }
  }

  void _scheduleReconnect() {
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(const Duration(seconds: 5), () {
      connect();
    });
  }

  Future<void> disconnect() async {
    _reconnectTimer?.cancel();
    await _channel?.sink.close();
    _isConnected = false;
  }

  void dispose() {
    disconnect();
    _controller.close();
  }
}

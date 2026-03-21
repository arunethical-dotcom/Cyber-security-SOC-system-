import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/incident.dart';
import '../services/api_service.dart';
import '../services/ws_service.dart';

final apiServiceProvider = Provider<ApiService>((ref) {
  return ApiService();
});

final wsServiceProvider = Provider<WsService>((ref) {
  final ws = WsService();
  ref.onDispose(() => ws.dispose());
  return ws;
});

final incidentProvider = StateNotifierProvider<IncidentNotifier, AsyncValue<List<Incident>>>((ref) {
  return IncidentNotifier(ref.watch(apiServiceProvider), ref.watch(wsServiceProvider));
});

class IncidentNotifier extends StateNotifier<AsyncValue<List<Incident>>> {
  final ApiService _apiService;
  final WsService _wsService;

  IncidentNotifier(this._apiService, this._wsService) : super(const AsyncValue.loading()) {
    _init();
  }

  Future<void> _init() async {
    await fetchIncidents();
    await _wsService.connect();
    _wsService.stream.listen((incident) {
      state.whenData((incidents) {
        state = AsyncValue.data([incident, ...incidents]);
      });
    });
  }

  Future<void> fetchIncidents({String? severity}) async {
    state = const AsyncValue.loading();
    try {
      final incidents = await _apiService.getIncidents(severity: severity);
      state = AsyncValue.data(incidents);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<void> refresh() async {
    await fetchIncidents();
  }
}

final selectedIncidentProvider = StateProvider<Incident?>((ref) => null);

final graphSnapshotProvider = FutureProvider<Map<String, dynamic>>((ref) async {
  final api = ref.watch(apiServiceProvider);
  return api.getGraphSnapshot();
});

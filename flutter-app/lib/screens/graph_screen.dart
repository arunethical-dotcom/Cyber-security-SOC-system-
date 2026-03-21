import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:fl_chart/fl_chart.dart';
import '../providers/incident_provider.dart';

class GraphScreen extends ConsumerWidget {
  const GraphScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final graphAsync = ref.watch(graphSnapshotProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Event Graph'),
      ),
      body: graphAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('Error: $e')),
        data: (graph) => _buildGraph(context, graph),
      ),
    );
  }

  Widget _buildGraph(BuildContext context, Map<String, dynamic> graph) {
    final nodes = graph['nodes'] as List? ?? [];
    final edges = graph['edges'] as List? ?? [];

    if (nodes.isEmpty) {
      return const Center(child: Text('No graph data available'));
    }

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Graph Overview',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 8),
          Text('${nodes.length} nodes, ${edges.length} edges'),
          const SizedBox(height: 24),
          Expanded(
            child: _buildNodeList(nodes),
          ),
        ],
      ),
    );
  }

  Widget _buildNodeList(List<dynamic> nodes) {
    return ListView.builder(
      itemCount: nodes.length,
      itemBuilder: (context, index) {
        final node = nodes[index] as Map<String, dynamic>;
        return Card(
          margin: const EdgeInsets.symmetric(vertical: 4),
          child: ListTile(
            leading: CircleAvatar(
              backgroundColor: _getKindColor(node['kind'] ?? ''),
              child: Icon(_getKindIcon(node['kind'] ?? '')),
            ),
            title: Text(node['key'] ?? 'Unknown'),
            subtitle: Text('Kind: ${node['kind'] ?? 'Unknown'}'),
          ),
        );
      },
    );
  }

  Color _getKindColor(String kind) {
    switch (kind.toUpperCase()) {
      case 'USER': return Colors.blue;
      case 'IP': return Colors.green;
      case 'DEVICE': return Colors.purple;
      case 'PROCESS': return Colors.orange;
      case 'FILE': return Colors.brown;
      default: return Colors.grey;
    }
  }

  IconData _getKindIcon(String kind) {
    switch (kind.toUpperCase()) {
      case 'USER': return Icons.person;
      case 'IP': return Icons.computer;
      case 'DEVICE': return Icons.devices;
      case 'PROCESS': return Icons.settings;
      case 'FILE': return Icons.insert_drive_file;
      default: return Icons.help_outline;
    }
  }
}

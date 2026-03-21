import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import '../providers/incident_provider.dart';
import '../models/incident.dart';
import 'alert_detail_screen.dart';

class DashboardScreen extends ConsumerStatefulWidget {
  const DashboardScreen({super.key});

  @override
  ConsumerState<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends ConsumerState<DashboardScreen> {
  String _selectedSeverity = 'ALL';

  @override
  Widget build(BuildContext context) {
    final incidentsAsync = ref.watch(incidentProvider);
    
    return Scaffold(
      appBar: AppBar(
        title: const Text('SOC Dashboard'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => ref.read(incidentProvider.notifier).refresh(),
          ),
        ],
      ),
      body: Column(
        children: [
          _buildSeverityFilter(),
          Expanded(
            child: incidentsAsync.when(
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Center(child: Text('Error: $e')),
              data: (incidents) {
                final filtered = _selectedSeverity == 'ALL'
                    ? incidents
                    : incidents.where((i) => i.severity.toUpperCase() == _selectedSeverity).toList();
                
                if (filtered.isEmpty) {
                  return const Center(child: Text('No incidents'));
                }
                
                return ListView.builder(
                  itemCount: filtered.length,
                  itemBuilder: (context, index) {
                    return _buildIncidentCard(filtered[index]);
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSeverityFilter() {
    return Container(
      padding: const EdgeInsets.all(8),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: ['ALL', 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL'].map((sev) {
            final isSelected = _selectedSeverity == sev;
            return Padding(
              padding: const EdgeInsets.symmetric(horizontal: 4),
              child: FilterChip(
                label: Text(sev),
                selected: isSelected,
                onSelected: (selected) {
                  setState(() => _selectedSeverity = sev);
                  ref.read(incidentProvider.notifier).fetchIncidents(
                    severity: sev == 'ALL' ? null : sev,
                  );
                },
                backgroundColor: _getSeverityColor(sev),
                selectedColor: _getSeverityColor(sev).withOpacity(0.8),
              ),
            );
          }).toList(),
        ),
      ),
    );
  }

  Widget _buildIncidentCard(Incident incident) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: _getSeverityColor(incident.severity),
          child: Text(incident.severity[0]),
        ),
        title: Text(incident.id.substring(0, 8)),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(DateFormat('yyyy-MM-dd HH:mm').format(incident.timestamp)),
            if (incident.chain.isNotEmpty)
              Text(incident.chain.join(', '), 
                style: const TextStyle(fontSize: 12)),
          ],
        ),
        trailing: const Icon(Icons.chevron_right),
        onTap: () {
          ref.read(selectedIncidentProvider.notifier).state = incident;
          Navigator.push(
            context,
            MaterialPageRoute(builder: (_) => const AlertDetailScreen()),
          );
        },
      ),
    );
  }

  Color _getSeverityColor(String severity) {
    switch (severity.toUpperCase()) {
      case 'CRITICAL': return Colors.red;
      case 'HIGH': return Colors.orange;
      case 'MEDIUM': return Colors.amber;
      case 'LOW': return Colors.green;
      default: return Colors.grey;
    }
  }
}

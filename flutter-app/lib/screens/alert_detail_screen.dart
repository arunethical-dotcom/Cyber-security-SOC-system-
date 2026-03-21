import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import '../providers/incident_provider.dart';
import 'report_screen.dart';
import 'feedback_screen.dart';

class AlertDetailScreen extends ConsumerWidget {
  const AlertDetailScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final incident = ref.watch(selectedIncidentProvider);
    
    if (incident == null) {
      return const Scaffold(
        body: Center(child: Text('No incident selected')),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: Text('Incident ${incident.id.substring(0, 8)}'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildSeverityBadge(incident.severity),
            const SizedBox(height: 16),
            Text(
              'Timestamp: ${DateFormat('yyyy-MM-dd HH:mm:ss').format(incident.timestamp)}',
              style: Theme.of(context).textTheme.bodyLarge,
            ),
            const SizedBox(height: 24),
            _buildSection('MITRE ATT&CK Chain', 
              incident.chain.map((t) => Chip(label: Text(t))).toList()),
            const SizedBox(height: 16),
            _buildSection('Affected Entities',
              incident.entities.map((e) => Chip(label: Text(e))).toList()),
            const SizedBox(height: 24),
            _buildScoreSection('Signal Scores', [
              _buildScoreBar('Sigma Score', incident.sigmaScore),
              const SizedBox(height: 8),
              _buildScoreBar('Z-Score', incident.zScore / 10),
              if (incident.iocMatch != null) ...[
                const SizedBox(height: 8),
                Text('IOC Match: ${incident.iocMatch}'),
              ],
            ]),
            const SizedBox(height: 24),
            if (incident.summary != null) ...[
              Text('LLM Summary', style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 8),
              Text(incident.summary!),
              const SizedBox(height: 16),
            ],
            if (incident.actions != null && incident.actions!.isNotEmpty) ...[
              Text('Recommended Actions', style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 8),
              ...incident.actions!.asMap().entries.map((e) => 
                Text('${e.key + 1}. ${e.value}')),
            ],
            const SizedBox(height: 32),
            Row(
              children: [
                ElevatedButton.icon(
                  icon: const Icon(Icons.description),
                  label: const Text('View Report'),
                  onPressed: () => Navigator.push(
                    context,
                    MaterialPageRoute(builder: (_) => ReportScreen(incidentId: incident.id)),
                  ),
                ),
                const SizedBox(width: 16),
                OutlinedButton.icon(
                  icon: const Icon(Icons.feedback),
                  label: const Text('Submit Feedback'),
                  onPressed: () => Navigator.push(
                    context,
                    MaterialPageRoute(builder: (_) => FeedbackScreen(incidentId: incident.id)),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSeverityBadge(String severity) {
    Color color;
    switch (severity.toUpperCase()) {
      case 'CRITICAL': color = Colors.red; break;
      case 'HIGH': color = Colors.orange; break;
      case 'MEDIUM': color = Colors.amber; break;
      default: color = Colors.green;
    }
    
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(20),
      ),
      child: Text(
        severity,
        style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold),
      ),
    );
  }

  Widget _buildSection(String title, List<Widget> children) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
        const SizedBox(height: 8),
        Wrap(spacing: 8, runSpacing: 4, children: children),
      ],
    );
  }

  Widget _buildScoreSection(String title, List<Widget> children) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold)),
        const SizedBox(height: 8),
        ...children,
      ],
    );
  }

  Widget _buildScoreBar(String label, double value) {
    return Row(
      children: [
        SizedBox(
          width: 100,
          child: Text(label),
        ),
        Expanded(
          child: LinearProgressIndicator(
            value: value.clamp(0.0, 1.0),
            backgroundColor: Colors.grey[300],
          ),
        ),
        const SizedBox(width: 8),
        Text(value.toStringAsFixed(2)),
      ],
    );
  }
}

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/incident_provider.dart';
import '../models/feedback.dart';

class FeedbackScreen extends ConsumerStatefulWidget {
  final String incidentId;

  const FeedbackScreen({super.key, required this.incidentId});

  @override
  ConsumerState<FeedbackScreen> createState() => _FeedbackScreenState();
}

class _FeedbackScreenState extends ConsumerState<FeedbackScreen> {
  final _formKey = GlobalKey<FormState>();
  String _action = 'confirm';
  final _entityController = TextEditingController();
  final _tacticController = TextEditingController();
  double _threshold = 0.5;
  bool _isSubmitting = false;

  @override
  void dispose() {
    _entityController.dispose();
    _tacticController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Submit Feedback'),
      ),
      body: Form(
        key: _formKey,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Incident: ${widget.incidentId.substring(0, 8)}',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 24),
              const Text('Action'),
              const SizedBox(height: 8),
              DropdownButtonFormField<String>(
                value: _action,
                items: const [
                  DropdownMenuItem(value: 'suppress', child: Text('Suppress')),
                  DropdownMenuItem(value: 'tune', child: Text('Tune')),
                  DropdownMenuItem(value: 'confirm', child: Text('Confirm')),
                ],
                onChanged: (value) => setState(() => _action = value!),
              ),
              const SizedBox(height: 16),
              TextFormField(
                controller: _entityController,
                decoration: const InputDecoration(
                  labelText: 'Entity (optional)',
                  border: OutlineInputBorder(),
                ),
              ),
              const SizedBox(height: 16),
              TextFormField(
                controller: _tacticController,
                decoration: const InputDecoration(
                  labelText: 'Tactic (optional)',
                  border: OutlineInputBorder(),
                ),
              ),
              if (_action == 'tune') ...[
                const SizedBox(height: 24),
                Text('New Threshold: ${_threshold.toStringAsFixed(2)}'),
                Slider(
                  value: _threshold,
                  min: 0.0,
                  max: 1.0,
                  divisions: 20,
                  onChanged: (value) => setState(() => _threshold = value),
                ),
              ],
              const SizedBox(height: 32),
              SizedBox(
                width: double.infinity,
                child: ElevatedButton(
                  onPressed: _isSubmitting ? null : _submitFeedback,
                  child: _isSubmitting
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Submit'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _submitFeedback() async {
    setState(() => _isSubmitting = true);

    try {
      final feedback = UserFeedback(
        incidentId: widget.incidentId,
        action: _action,
        entity: _entityController.text.isEmpty ? null : _entityController.text,
        tactic: _tacticController.text.isEmpty ? null : _tacticController.text,
        newThreshold: _action == 'tune' ? _threshold : null,
      );

      final api = ref.read(apiServiceProvider);
      await api.submitFeedback(feedback);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Feedback submitted successfully')),
        );
        Navigator.pop(context);
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isSubmitting = false);
      }
    }
  }
}

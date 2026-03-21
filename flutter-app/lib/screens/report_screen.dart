import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:pdfx/pdfx.dart';
import '../providers/incident_provider.dart';

class ReportScreen extends ConsumerStatefulWidget {
  final String incidentId;

  const ReportScreen({super.key, required this.incidentId});

  @override
  ConsumerState<ReportScreen> createState() => _ReportScreenState();
}

class _ReportScreenState extends ConsumerState<ReportScreen> {
  PdfController? _pdfController;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadPdf();
  }

  Future<void> _loadPdf() async {
    try {
      final api = ref.read(apiServiceProvider);
      final pdfBytes = await api.getReportPdf(widget.incidentId);
      
      final document = await PdfDocument.openData(pdfBytes);
      
      if (mounted) {
        setState(() {
          _pdfController = PdfController(document: Future.value(document));
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _isLoading = false;
        });
      }
    }
  }

  @override
  void dispose() {
    _pdfController?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Report ${widget.incidentId.substring(0, 8)}'),
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 16),
            Text('Error loading report: $_error'),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () {
                setState(() {
                  _isLoading = true;
                  _error = null;
                });
                _loadPdf();
              },
              child: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    if (_pdfController == null) {
      return const Center(child: Text('No PDF available'));
    }

    return PdfView(
      controller: _pdfController!,
      builders: PdfViewBuilders<DefaultBuilderOptions>(
        options: const DefaultBuilderOptions(),
      ),
    );
  }
}

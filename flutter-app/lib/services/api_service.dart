import 'dart:typed_data';
import 'package:dio/dio.dart';
import '../models/incident.dart';
import '../models/feedback.dart';

class ApiService {
  static const String baseUrl = 'http://localhost:8080';
  final Dio _dio;

  ApiService() : _dio = Dio(BaseOptions(
    baseUrl: baseUrl,
    connectTimeout: const Duration(seconds: 10),
    receiveTimeout: const Duration(seconds: 30),
  ));

  Future<List<Incident>> getIncidents({
    int page = 1,
    int limit = 50,
    String? severity,
  }) async {
    try {
      final response = await _dio.get(
        '/incidents',
        queryParameters: {
          'page': page,
          'limit': limit,
          if (severity != null) 'severity': severity,
        },
      );
      
      final List<dynamic> data = response.data;
      return data.map((json) => Incident.fromJson(json)).toList();
    } catch (e) {
      throw Exception('Failed to fetch incidents: $e');
    }
  }

  Future<Incident> getIncidentById(String id) async {
    try {
      final response = await _dio.get('/incidents/$id');
      return Incident.fromJson(response.data);
    } catch (e) {
      throw Exception('Failed to fetch incident: $e');
    }
  }

  Future<Uint8List> getReportPdf(String id) async {
    try {
      final response = await _dio.get(
        '/reports/$id/pdf',
        options: Options(responseType: ResponseType.bytes),
      );
      return Uint8List.fromList(response.data);
    } catch (e) {
      throw Exception('Failed to fetch report PDF: $e');
    }
  }

  Future<void> submitFeedback(UserFeedback feedback) async {
    try {
      await _dio.post(
        '/feedback',
        data: feedback.toJson(),
      );
    } catch (e) {
      throw Exception('Failed to submit feedback: $e');
    }
  }

  Future<Map<String, dynamic>> getGraphSnapshot() async {
    try {
      final response = await _dio.get('/graph/snapshot');
      return response.data as Map<String, dynamic>;
    } catch (e) {
      throw Exception('Failed to fetch graph snapshot: $e');
    }
  }
}

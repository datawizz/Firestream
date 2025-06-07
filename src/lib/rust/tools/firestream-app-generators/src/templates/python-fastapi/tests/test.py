#!/usr/bin/env python3
"""
Test suite for the FastAPI application
"""

import pytest
from fastapi.testclient import TestClient
from src.main import app

client = TestClient(app)


def test_root():
    """Test root endpoint"""
    response = client.get("/")
    assert response.status_code == 200
    assert response.json() == {"message": "Welcome to FastAPI service!"}


def test_health():
    """Test health endpoint"""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert data["service"] == "fastapi-service"
    assert data["version"] == "1.0.0"


def test_hello():
    """Test hello endpoint"""
    response = client.get("/api/v1/hello/World")
    assert response.status_code == 200
    assert response.json() == {"message": "Hello, World!"}


def test_hello_empty_name():
    """Test hello endpoint with empty name"""
    response = client.get("/api/v1/hello/")
    assert response.status_code == 404  # FastAPI returns 404 for missing path parameter


if __name__ == "__main__":
    pytest.main([__file__])

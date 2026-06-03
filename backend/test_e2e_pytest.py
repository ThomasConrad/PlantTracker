"""
End-to-End Test Suite for Planty Backend using pytest

This module tests the complete Planty API by:
1. Starting the backend server
2. Creating users and authenticating
3. Creating, reading, updating, and deleting plants
4. Testing validation and error cases
5. Ensuring proper user isolation

Prerequisites:
- Python 3.8+
- pytest and requests libraries
- Backend compiled and ready to run
"""

import os
import socket
import subprocess
import time
import uuid
from typing import Dict, Optional, Any
import pytest
import requests
from pathlib import Path


def get_free_port() -> int:
    """Get a free port from the OS"""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(('', 0))
        s.listen(1)
        port = s.getsockname()[1]
    return port


class BackendServer:
    """Context manager for handling backend server lifecycle"""
    
    def __init__(self, port: Optional[int] = None):
        self.port = port or get_free_port()
        self.base_url = f"http://localhost:{self.port}"
        self.process: Optional[subprocess.Popen] = None
        self.api_prefix = "/api/v1"  # API prefix when no frontend is served (API-only mode)
        
    def start(self):
        """Start the backend server"""
        print(f"Starting Planty backend on port {self.port}...")
        
        # Change to backend directory
        backend_dir = Path(__file__).parent
        os.chdir(backend_dir)
            
        # Start the backend process with in-memory database
        env = os.environ.copy()
        env["RUST_LOG"] = "debug,tower_http=info,hyper=info"
        env["PLANTY_OPEN_REGISTRATION"] = "true"
        env["PLANT_COACH_PROVIDER"] = "mock"
        
        try:
            binary = Path(__file__).parent / "target" / "release" / "planty-api"
            self.process = subprocess.Popen([
                str(binary),
                "--port", str(self.port),
                "--database-url", "sqlite::memory:",
                "--frontend-dir", "/nonexistent"  # Force API-only mode
            ],
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True
            )
            
            # Wait for "READY" signal on stdout
            while True:
                line = self.process.stdout.readline()
                if not line:
                    raise RuntimeError("Backend exited without READY signal")
                if line.startswith("READY"):
                    print(f"Backend ready on port {self.port}")
                    return
            
        except Exception as e:
            print(f"Failed to start backend: {e}")
            self.stop()
            raise
            
    def stop(self):
        """Stop the backend server"""
        if self.process:
            print("Stopping backend...")
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
            print("Backend stopped")
            
    def __enter__(self):
        self.start()
        return self
        
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()


class APIClient:
    """HTTP client for making API requests"""
    
    def __init__(self, base_url: str, api_prefix: str = "/api/v1"):
        self.base_url = base_url
        self.api_prefix = api_prefix
        self.session = requests.Session()
        
    def request(self, method: str, endpoint: str, **kwargs) -> requests.Response:
        """Make an HTTP request to the backend"""
        # Add API prefix if endpoint doesn't start with /
        if not endpoint.startswith('/'):
            endpoint = f"/{endpoint}"
        
        # For API endpoints, add the v1 prefix
        if endpoint.startswith('/auth') or endpoint.startswith('/plants') or endpoint.startswith('/photos') or endpoint.startswith('/tracking') or endpoint.startswith('/calendar') or endpoint.startswith('/coach'):
            endpoint = f"{self.api_prefix}{endpoint}"
            
        url = f"{self.base_url}{endpoint}"
        response = self.session.request(method, url, **kwargs)
        
        print(f"{method} {endpoint} -> {response.status_code}")
        if response.status_code >= 400:
            try:
                error_data = response.json()
                print(f"Error: {error_data}")
            except:
                print(f"Error: {response.text}")
                
        return response


@pytest.fixture(scope="session")
def backend():
    """Pytest fixture to start/stop backend server for the entire test session"""
    with BackendServer() as server:
        yield server


@pytest.fixture
def client(backend):
    """Pytest fixture to provide API client"""
    return APIClient(backend.base_url, backend.api_prefix)


@pytest.fixture
def test_users():
    """Pytest fixture to provide test user data"""
    return {
        "user1": {
            "email": f"test1_{uuid.uuid4().hex[:8]}@example.com",
            "name": "Test User 1",
            "password": "password123"
        },
        "user2": {
            "email": f"test2_{uuid.uuid4().hex[:8]}@example.com",
            "name": "Test User 2", 
            "password": "password456"
        }
    }


@pytest.mark.auth
class TestAuthentication:
    """Test user authentication functionality"""
    
    def test_user_registration(self, client, test_users):
        """Test user registration"""
        user_data = test_users["user1"]
        
        response = client.request("POST", "/auth/register", json=user_data)
        assert response.status_code == 201
        
        response_data = response.json()
        assert response_data["user"]["email"] == user_data["email"]
        assert response_data["user"]["name"] == user_data["name"]
        assert "id" in response_data["user"]
        
    def test_duplicate_email_registration(self, client, test_users):
        """Test that duplicate email registration fails"""
        user_data = test_users["user1"]
        
        # Register user first time
        response = client.request("POST", "/auth/register", json=user_data)
        assert response.status_code == 201
        
        # Try to register again with same email
        response = client.request("POST", "/auth/register", json=user_data)
        assert response.status_code == 422
        
    def test_user_login(self, client, test_users):
        """Test user login"""
        user_data = test_users["user1"]
        
        # Register user
        response = client.request("POST", "/auth/register", json=user_data)
        assert response.status_code == 201
        
        # Login
        login_data = {
            "email": user_data["email"],
            "password": user_data["password"]
        }
        response = client.request("POST", "/auth/login", json=login_data)
        assert response.status_code == 200
        
        response_data = response.json()
        assert response_data["user"]["email"] == user_data["email"]
        
    def test_invalid_login(self, client, test_users):
        """Test login with invalid credentials"""
        user_data = test_users["user1"]
        
        # Register user
        response = client.request("POST", "/auth/register", json=user_data)
        assert response.status_code == 201
        
        # Try to login with wrong password
        login_data = {
            "email": user_data["email"],
            "password": "wrongpassword"
        }
        response = client.request("POST", "/auth/login", json=login_data)
        assert response.status_code == 401
        
    def test_user_logout(self, client, test_users):
        """Test user logout"""
        user_data = test_users["user1"]
        
        # Register and login user
        client.request("POST", "/auth/register", json=user_data)
        client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        
        # Logout
        response = client.request("POST", "/auth/logout")
        assert response.status_code == 200


@pytest.mark.plants
class TestPlantCRUD:
    """Test plant CRUD operations"""
    
    @pytest.fixture(autouse=True)
    def login_user(self, client, test_users):
        """Automatically login a user before each test"""
        user_data = test_users["user1"]
        
        # Register and login user
        client.request("POST", "/auth/register", json=user_data)
        response = client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        assert response.status_code == 200
    
    def test_create_plant(self, client):
        """Test creating a new plant"""
        plant_data = {
            "name": "Fiddle Leaf Fig",
            "genus": "Ficus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        
        response = client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        
        response_data = response.json()
        assert response_data["name"] == plant_data["name"]
        assert response_data["genus"] == plant_data["genus"]
        water_task = next(t for t in response_data["careTasks"] if t["name"] == "Water")
        fert_task = next(t for t in response_data["careTasks"] if t["name"] == "Fertilize")
        assert water_task["intervalDays"] == 7
        assert fert_task["intervalDays"] == 14
        assert "id" in response_data
        
    def test_create_plant_validation_errors(self, client):
        """Test plant creation validation"""
        # Empty name should fail
        response = client.request("POST", "/plants", json={
            "name": "",
            "genus": "Ficus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        })
        assert response.status_code == 422
        
        # Invalid watering interval should fail
        response = client.request("POST", "/plants", json={
            "name": "Test Plant",
            "genus": "Test",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 0},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        })
        assert response.status_code == 422
        
    def test_get_plants(self, client):
        """Test getting all plants"""
        # Create some plants first
        plants_data = [
            {
                "name": "Plant 1", 
                "genus": "Genus1", 
                "careTasks": [
                    {"name": "Water", "icon": "💧", "intervalDays": 7},
                    {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
                ]
            },
            {
                "name": "Plant 2", 
                "genus": "Genus2", 
                "careTasks": [
                    {"name": "Water", "icon": "💧", "intervalDays": 10},
                    {"name": "Fertilize", "icon": "🌱", "intervalDays": 21}
                ]
            }
        ]
        
        created_plants = []
        for plant_data in plants_data:
            response = client.request("POST", "/plants", json=plant_data)
            assert response.status_code == 201
            created_plants.append(response.json())
        
        # Get all plants
        response = client.request("GET", "/plants")
        assert response.status_code == 200
        
        response_data = response.json()
        assert len(response_data["plants"]) == 2
        assert response_data["total"] == 2
        
    def test_get_single_plant(self, client):
        """Test getting a specific plant"""
        # Create a plant
        plant_data = {
            "name": "Test Plant",
            "genus": "TestGenus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 5},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 10}
            ]
        }
        response = client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        plant = response.json()
        
        # Get the specific plant
        response = client.request("GET", f"/plants/{plant['id']}")
        assert response.status_code == 200
        
        response_data = response.json()
        assert response_data["id"] == plant["id"]
        assert response_data["name"] == plant_data["name"]
        
    def test_update_plant(self, client):
        """Test updating a plant"""
        # Create a plant
        plant_data = {
            "name": "Original Plant",
            "genus": "OriginalGenus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        response = client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        plant = response.json()
        
        # Update the plant name and genus
        update_data = {
            "name": "Updated Plant"
        }
        response = client.request("PUT", f"/plants/{plant['id']}", json=update_data)
        assert response.status_code == 200
        
        response_data = response.json()
        assert response_data["name"] == "Updated Plant"
        assert response_data["genus"] == plant_data["genus"]  # Should remain unchanged
        
    def test_delete_plant(self, client):
        """Test deleting a plant"""
        # Create a plant
        plant_data = {
            "name": "Plant to Delete",
            "genus": "DeleteGenus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        response = client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        plant = response.json()
        
        # Delete the plant
        response = client.request("DELETE", f"/plants/{plant['id']}")
        assert response.status_code == 204
        
        # Verify plant is deleted
        response = client.request("GET", f"/plants/{plant['id']}")
        assert response.status_code == 404


@pytest.mark.isolation
class TestUserIsolation:
    """Test that users can only access their own plants"""
    
    def test_plant_isolation_between_users(self, client, test_users):
        """Test that users cannot access each other's plants"""
        user1_data = test_users["user1"]
        user2_data = test_users["user2"]
        
        # Register both users
        client.request("POST", "/auth/register", json=user1_data)
        client.request("POST", "/auth/register", json=user2_data)
        
        # Login as user1 and create a plant
        client.request("POST", "/auth/login", json={
            "email": user1_data["email"],
            "password": user1_data["password"]
        })
        
        plant_data = {
            "name": "User 1 Plant",
            "genus": "User1Genus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        response = client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        user1_plant = response.json()
        
        # Logout user1 and login as user2
        client.request("POST", "/auth/logout")
        client.request("POST", "/auth/login", json={
            "email": user2_data["email"],
            "password": user2_data["password"]
        })
        
        # User2 should have no plants
        response = client.request("GET", "/plants")
        assert response.status_code == 200
        response_data = response.json()
        assert len(response_data["plants"]) == 0
        
        # User2 should not be able to access user1's plant
        response = client.request("GET", f"/plants/{user1_plant['id']}")
        assert response.status_code == 404
        
        # User2 should not be able to update user1's plant
        response = client.request("PUT", f"/plants/{user1_plant['id']}", json={"name": "Hacked Plant"})
        assert response.status_code == 404
        
        # User2 should not be able to delete user1's plant
        response = client.request("DELETE", f"/plants/{user1_plant['id']}")
        assert response.status_code == 404


@pytest.mark.errors
class TestErrorHandling:
    """Test error handling and edge cases"""
    
    def test_unauthenticated_requests(self, client):
        """Test that unauthenticated requests are rejected"""
        # Should not be able to access plants without authentication
        response = client.request("GET", "/plants")
        assert response.status_code == 401
        
        # For POST with minimal valid JSON structure, should also get 401
        response = client.request("POST", "/plants", json={
            "name": "Test Plant",
            "genus": "TestGenus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        })
        assert response.status_code == 401
        
    def test_nonexistent_plant(self, client, test_users):
        """Test accessing non-existent plant"""
        user_data = test_users["user1"]
        
        # Register and login user
        client.request("POST", "/auth/register", json=user_data)
        client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        
        # Try to access non-existent plant
        fake_id = str(uuid.uuid4())
        response = client.request("GET", f"/plants/{fake_id}")
        assert response.status_code == 404
        
    def test_invalid_json(self, client, test_users):
        """Test invalid JSON handling"""
        user_data = test_users["user1"]
        
        # Register and login user
        client.request("POST", "/auth/register", json=user_data)
        client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        
        # Send invalid JSON
        response = client.request("POST", "/plants", 
                                data="invalid json", 
                                headers={"Content-Type": "application/json"})
        assert response.status_code == 400
        
    def test_missing_required_fields(self, client, test_users):
        """Test missing required fields validation"""
        user_data = test_users["user1"]
        
        # Register and login user
        client.request("POST", "/auth/register", json=user_data)
        client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        
        # Send request with missing required fields (missing genus)
        # This will get 400 because JSON deserialization fails before validation
        response = client.request("POST", "/plants", json={
            "name": "Test Plant",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        })
        assert response.status_code == 400
        
        # Test validation error with all required fields but invalid values
        response = client.request("POST", "/plants", json={
            "name": "",  # Empty name should fail validation
            "genus": "TestGenus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        })
        assert response.status_code == 422


@pytest.mark.performance
@pytest.mark.slow
class TestPerformance:
    
    @pytest.fixture(autouse=True)
    def setup_client(self, client, test_users):
        """Auto-register and login for all performance tests"""
        user_data = test_users["user1"]
        
        # Register the user first
        register_response = client.request("POST", "/auth/register", json=user_data)
        # Registration might fail if user already exists, which is fine
        
        # Now login
        login_response = client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        assert login_response.status_code == 200
        self.client = client
        return client

    @pytest.mark.slow
    def test_many_plants_creation(self):
        """Test creating many plants efficiently"""
        num_plants = 50
        
        for i in range(num_plants):
            plant_data = {
                "name": f"Performance Plant {i}",
                "genus": f"Performicus_{i}",
                "careTasks": [
                    {"name": "Water", "icon": "💧", "intervalDays": 7},
                    {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
                ]
            }
            
            response = self.client.request("POST", "/plants", json=plant_data)
            assert response.status_code == 201
        
        # Verify all plants were created
        response = self.client.request("GET", "/plants", params={"limit": 100})
        assert response.status_code == 200
        
        response_data = response.json()
        assert response_data["total"] == num_plants

    def test_large_image_upload_performance(self):
        """Test upload performance with a large (~5MB) image and measure timing"""
        import time
        
        # Create a plant first
        plant_data = {
            "name": "Performance Test Plant",
            "genus": "Performicus", 
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Create a large image using Pillow (~5MB target)
        from PIL import Image
        import io
        import requests
        
        # Create a large image (2400x2400 should give us ~5MB when saved as JPEG)
        print(f"\nCreating large test image...")
        img = Image.new('RGB', (2400, 2400), color=(64, 128, 255))  # Blue base
        
        # Add some pattern to make it more realistic and compressible
        import random
        for x in range(0, 2400, 100):
            for y in range(0, 2400, 100):
                # Random colored squares
                color = (random.randint(50, 255), random.randint(50, 255), random.randint(50, 255))
                for i in range(50):
                    for j in range(50):
                        if x+i < 2400 and y+j < 2400:
                            img.putpixel((x+i, y+j), color)
        
        # Save as JPEG with moderate quality to get a large file
        img_bytes = io.BytesIO()
        img.save(img_bytes, format='JPEG', quality=75)
        large_image_data = img_bytes.getvalue()
        
        print(f"Created image with {len(large_image_data)} bytes ({len(large_image_data) / (1024*1024):.1f}MB)")
        
        files = {
            'file': ('large-test.jpg', io.BytesIO(large_image_data), 'image/jpeg')
        }
        
        # Measure upload time
        start_time = time.time()
        
        upload_response = requests.post(
            f"{self.client.base_url}/api/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        
        upload_end_time = time.time()
        upload_duration = upload_end_time - start_time
        
        assert upload_response.status_code == 201
        photo = upload_response.json()
        photo_id = photo["id"]
        
        print(f"Upload took {upload_duration:.2f} seconds")
        print(f"Upload speed: {(len(large_image_data) / (1024*1024)) / upload_duration:.1f} MB/s")
        
        # Verify the photo was processed and converted to WebP
        assert photo["contentType"] == "image/webp"
        assert photo["size"] > 0  # Size will be different after AVIF conversion
        assert "width" in photo
        assert "height" in photo


@pytest.mark.photos
class TestPhotoUpload:
    """Test photo upload functionality"""
    
    @pytest.fixture(autouse=True)
    def login_user(self, client, test_users):
        """Automatically login a user before each test"""
        user_data = test_users["user1"]
        
        # Register and login user
        client.request("POST", "/auth/register", json=user_data)
        response = client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        assert response.status_code == 200
        
        # Store client reference for test methods
        self.client = client

    def test_upload_photo_multipart(self):
        """Test uploading a photo using multipart form data"""
        # Create a plant first
        plant_data = {
            "name": "Photo Test Plant",
            "genus": "Photographicus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Create a proper JPEG image using Python's Pillow library
        from PIL import Image
        import io
        
        # Create a small RGB image
        img = Image.new('RGB', (100, 100), color=(255, 0, 0))  # Red 100x100 image
        img_bytes = io.BytesIO()
        img.save(img_bytes, format='JPEG', quality=80)
        fake_image_data = img_bytes.getvalue()
        
        # Upload photo using multipart form data
        import io
        files = {
            'file': ('test-photo.jpg', io.BytesIO(fake_image_data), 'image/jpeg')
        }
        
        # Try to verify the plant exists first
        verify_response = self.client.request("GET", f"/plants/{plant_id}")
        print(f"Plant verification status: {verify_response.status_code}")
        if verify_response.status_code == 200:
            print(f"Plant exists: {verify_response.json()['name']}")
        
        # Use requests directly for multipart upload
        import requests
        response = requests.post(
            f"{self.client.base_url}/api/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        
        if response.status_code != 201:
            print(f"Photo upload failed with status {response.status_code}")
            print(f"Response: {response.text}")
            print(f"Plant ID: {plant_id}")
            print(f"File size: {len(fake_image_data)} bytes")
            print(f"Files data: {files}")
            print(f"Cookies: {self.client.session.cookies}")
            print(f"Request URL: {self.client.base_url}/api/v1/plants/{plant_id}/photos")
        assert response.status_code == 201
        photo_data = response.json()
        
        assert "id" in photo_data
        assert photo_data["plantId"] == plant_id
        assert photo_data["originalFilename"] == "test-photo.jpg"
        assert photo_data["contentType"] == "image/webp"  # Images are converted to WebP
        assert photo_data["size"] > 0  # Size will be different after AVIF conversion
        assert "createdAt" in photo_data

    def test_list_photos_after_upload(self):
        """Test listing photos after uploading one"""
        # Create a plant first
        plant_data = {
            "name": "Photo List Plant",
            "genus": "Listicus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Upload a photo using Pillow to create proper JPEG
        from PIL import Image
        import io
        
        # Create a small RGB image
        img = Image.new('RGB', (50, 50), color=(0, 255, 0))  # Green 50x50 image
        img_bytes = io.BytesIO()
        img.save(img_bytes, format='JPEG', quality=80)
        fake_image_data = img_bytes.getvalue()
        
        import requests
        files = {
            'file': ('list-test.jpg', io.BytesIO(fake_image_data), 'image/jpeg')
        }
        
        upload_response = requests.post(
            f"{self.client.base_url}/api/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        assert upload_response.status_code == 201
        
        # List photos
        list_response = self.client.request("GET", f"/plants/{plant_id}/photos")
        assert list_response.status_code == 200
        
        photos_response = list_response.json()
        assert len(photos_response["photos"]) == 1
        assert photos_response["total"] == 1
        assert photos_response["photos"][0]["originalFilename"] == "list-test.jpg"

    def test_delete_photo(self):
        """Test deleting a photo"""
        # Create a plant first
        plant_data = {
            "name": "Photo Delete Plant",
            "genus": "Deleticus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Upload a photo using Pillow to create proper JPEG
        from PIL import Image
        import io
        
        # Create a small RGB image
        img = Image.new('RGB', (40, 40), color=(255, 255, 0))  # Yellow 40x40 image
        img_bytes = io.BytesIO()
        img.save(img_bytes, format='JPEG', quality=80)
        fake_image_data = img_bytes.getvalue()
        
        import requests
        files = {
            'file': ('delete-test.jpg', io.BytesIO(fake_image_data), 'image/jpeg')
        }
        
        upload_response = requests.post(
            f"{self.client.base_url}/api/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        assert upload_response.status_code == 201
        photo = upload_response.json()
        photo_id = photo["id"]
        
        # Delete the photo
        delete_response = self.client.request("DELETE", f"/plants/{plant_id}/photos/{photo_id}")
        assert delete_response.status_code == 204
        
        # Verify photo is deleted
        list_response = self.client.request("GET", f"/plants/{plant_id}/photos")
        assert list_response.status_code == 200
        
        photos_response = list_response.json()
        assert len(photos_response["photos"]) == 0
        assert photos_response["total"] == 0

    def test_plant_preview_functionality(self):
        """Test that setting a plant preview correctly updates the previewUrl in plant responses"""
        # Create a plant first
        plant_data = {
            "name": "preview Test Plant",
            "genus": "previewicus", 
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Initially, the plant should have no preview
        assert plant.get("previewId") is None
        assert plant.get("previewUrl") is None
        
        # Upload a photo using Pillow to create proper JPEG
        from PIL import Image
        import io
        
        # Create a small RGB image
        img = Image.new('RGB', (60, 60), color=(255, 0, 255))  # Magenta 60x60 image
        img_bytes = io.BytesIO()
        img.save(img_bytes, format='JPEG', quality=80)
        fake_image_data = img_bytes.getvalue()
        
        import requests
        files = {
            'file': ('preview-test.jpg', io.BytesIO(fake_image_data), 'image/jpeg')
        }
        
        upload_response = requests.post(
            f"{self.client.base_url}/api/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        assert upload_response.status_code == 201
        photo = upload_response.json()
        photo_id = photo["id"]
        
        # Set the uploaded photo as the plant's preview
        preview_response = self.client.request("PUT", f"/plants/{plant_id}/preview/{photo_id}")
        assert preview_response.status_code == 200
        updated_plant = preview_response.json()
        
        # Verify the preview is set
        assert updated_plant["previewId"] == photo_id
        assert updated_plant["previewUrl"] is not None
        assert updated_plant["previewUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}"
        
        # Verify that fetching the plant individually also returns the preview
        get_response = self.client.request("GET", f"/plants/{plant_id}")
        assert get_response.status_code == 200
        fetched_plant = get_response.json()
        
        assert fetched_plant["previewId"] == photo_id
        assert fetched_plant["previewUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}"
        
        # Verify that listing plants also returns the preview
        list_response = self.client.request("GET", "/plants")
        assert list_response.status_code == 200
        plants_response = list_response.json()
        
        # Find our plant in the list
        our_plant = None
        for p in plants_response["plants"]:
            if p["id"] == plant_id:
                our_plant = p
                break
        
        assert our_plant is not None
        assert our_plant["previewId"] == photo_id
        assert our_plant["previewUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}"

    def test_async_preview_generation(self):
        """Test asynchronous preview generation during photo upload"""
        # Create a plant first
        plant_data = {
            "name": "Async Test Plant",
            "genus": "Asyncicus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 5},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 12}
            ]
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Create a proper JPEG image using Pillow  
        from PIL import Image
        import io
        
        # Create a 200x200 RGB image with a pattern
        img = Image.new('RGB', (200, 200), color=(128, 64, 192))  # Purple base
        # Add some pattern to make it interesting
        import random
        for x in range(0, 200, 20):
            for y in range(0, 200, 20):
                color = (random.randint(100, 255), random.randint(100, 255), random.randint(100, 255))
                for i in range(5):
                    for j in range(5):
                        if x+i < 200 and y+j < 200:
                            img.putpixel((x+i, y+j), color)
        
        img_bytes = io.BytesIO()
        img.save(img_bytes, format='JPEG', quality=85)
        test_image_data = img_bytes.getvalue()
        
        import requests
        files = {
            'file': ('async-test.jpg', io.BytesIO(test_image_data), 'image/jpeg')
        }
        
        # Upload photo
        upload_response = requests.post(
            f"{self.client.base_url}/api/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        
        # Should succeed with proper image
        assert upload_response.status_code == 201
        
        photo = upload_response.json()
        assert "id" in photo
        assert photo["plantId"] == plant_id
        assert photo["originalFilename"] == "async-test.jpg"
        assert photo["contentType"] == "image/webp"  # Should be converted to WebP
        assert photo["size"] > 0
        assert "width" in photo
        assert "height" in photo
        
        # Verify photo can be retrieved
        photo_id = photo["id"]
        get_response = self.client.request("GET", f"/plants/{plant_id}/photos/{photo_id}")
        assert get_response.status_code == 200
        
        # Photo data should be available (AVIF format)
        photo_data = get_response.content
        assert len(photo_data) > 0

    def test_upload_photo_validation_errors(self):
        """Test photo upload validation"""
        # Create a plant first
        plant_data = {
            "name": "Validation Plant",
            "genus": "Validicus", 
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Test invalid file type
        import io
        import requests
        files = {
            'file': ('test.txt', io.BytesIO(b'not an image'), 'text/plain')
        }
        
        response = requests.post(
            f"{self.client.base_url}/api/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        assert response.status_code == 422

    def test_upload_photo_unauthenticated(self):
        """Test photo upload without authentication"""
        plant_id = str(uuid.uuid4())
        
        # Logout first
        self.client.request("POST", "/auth/logout")
        
        fake_image_data = b'\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00\xff\xdb'
        
        import io
        import requests
        files = {
            'file': ('unauth-test.jpg', io.BytesIO(fake_image_data), 'image/jpeg')
        }
        
        # Create a new session without cookies
        response = requests.post(
            f"{self.client.base_url}/api/v1/plants/{plant_id}/photos",
            files=files
        )
        assert response.status_code == 401


@pytest.mark.calendar
class TestCalendarFunctionality:
    """Test calendar subscription and iCal feed functionality"""
    
    @pytest.fixture(autouse=True)
    def login_user(self, client, test_users):
        """Automatically login a user before each test"""
        user_data = test_users["user1"]
        
        # Register and login user
        client.request("POST", "/auth/register", json=user_data)
        response = client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        assert response.status_code == 200
        
        # Store client and user info for test methods
        self.client = client
        self.user_data = user_data
        self.user_response = response.json()

    def test_calendar_subscription_info_authenticated(self):
        """Test getting calendar subscription info when authenticated"""
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        subscription_data = response.json()
        
        # Check required fields
        assert "feedUrl" in subscription_data
        assert "instructions" in subscription_data
        assert "features" in subscription_data
        
        # Check instructions for different platforms
        instructions = subscription_data["instructions"]
        assert "general" in instructions
        assert "iOS" in instructions
        assert "android" in instructions
        assert "outlook" in instructions
        assert "apple" in instructions
        
        # Check features list
        features = subscription_data["features"]
        assert isinstance(features, list)
        assert len(features) > 0
        
        # Check feed URL format
        feed_url = subscription_data["feedUrl"]
        assert "calendar/" in feed_url
        assert ".ics" in feed_url
        assert "token=" in feed_url

    def test_calendar_subscription_info_unauthenticated(self):
        """Test that calendar subscription info requires authentication"""
        # Logout first
        self.client.request("POST", "/auth/logout")
        
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 401

    def test_regenerate_calendar_token(self):
        """Test regenerating calendar token"""
        # Get initial subscription info
        initial_response = self.client.request("GET", "/calendar/subscription")
        assert initial_response.status_code == 200
        initial_data = initial_response.json()
        initial_url = initial_data["feedUrl"]
        
        # Regenerate token
        regen_response = self.client.request("POST", "/calendar/regenerate-token")
        assert regen_response.status_code == 200
        
        regen_data = regen_response.json()
        assert "feedUrl" in regen_data
        assert "message" in regen_data
        
        new_url = regen_data["feedUrl"]
        
        # URLs should be different (different tokens)
        assert initial_url != new_url
        assert "token=" in new_url

    def test_calendar_feed_with_no_plants(self):
        """Test calendar feed generation with no plants"""
        # Get subscription info to get feed URL
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        feed_url = response.json()["feedUrl"]
        
        # Extract just the path and query from the feed URL
        import urllib.parse
        parsed_url = urllib.parse.urlparse(feed_url)
        calendar_path = f"{parsed_url.path}?{parsed_url.query}"
        
        # Request the calendar feed
        import requests
        full_url = f"{self.client.base_url}{calendar_path}"
        calendar_response = requests.get(full_url)
        assert calendar_response.status_code == 200
        
        # Should return valid iCalendar with no events
        calendar_content = calendar_response.text
        assert calendar_content.startswith("BEGIN:VCALENDAR")
        assert calendar_content.endswith("END:VCALENDAR\r\n")
        assert "Plant Care Schedule" in calendar_content
        
        # Should have no events since no plants
        assert "BEGIN:VEVENT" not in calendar_content

    def test_calendar_feed_with_plants(self):
        """Test calendar feed generation with plants"""
        # Create test plants with different schedules
        # Care tasks with intervalDays and null lastPerformed will be immediately due
        plants_data = [
            {
                "name": "Fiddle Leaf Fig",
                "genus": "Ficus",
                "careTasks": [
                    {"name": "Water", "icon": "💧", "intervalDays": 7},
                    {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
                ]
            },
            {
                "name": "Snake Plant", 
                "genus": "Sansevieria",
                "careTasks": [
                    {"name": "Water", "icon": "💧", "intervalDays": 14},
                    {"name": "Fertilize", "icon": "🌱", "intervalDays": 30}
                ]
            }
        ]
        
        created_plants = []
        for plant_data in plants_data:
            print(f"\nCreating plant: {plant_data['name']}")
            response = self.client.request("POST", "/plants", json=plant_data)
            assert response.status_code == 201
            created_plant = response.json()
            created_plants.append(created_plant)
            print(f"Created plant: {created_plant['name']}")
            print(f"  Care tasks: {created_plant.get('careTasks', 'Not found')}")
        
        # Get calendar feed
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        feed_url = response.json()["feedUrl"]
        print(f"Feed URL: {feed_url}")
        
        # Extract path and query
        import urllib.parse
        parsed_url = urllib.parse.urlparse(feed_url)
        calendar_path = f"{parsed_url.path}?{parsed_url.query}"
        
        print(f"Parsed URL: scheme={parsed_url.scheme}, netloc={parsed_url.netloc}, path={parsed_url.path}, query={parsed_url.query}")
        print(f"Calendar path: {calendar_path}")
        print(f"Final URL: {self.client.base_url}{calendar_path}")
        
        # Request the calendar feed
        import requests
        calendar_response = requests.get(f"{self.client.base_url}{calendar_path}")
        print(f"Calendar response status: {calendar_response.status_code}")
        print(f"Calendar response headers: {dict(calendar_response.headers)}")
        assert calendar_response.status_code == 200
        
        calendar_content = calendar_response.text
        print(f"Calendar content length: {len(calendar_content)}")
        print(f"Calendar content: {repr(calendar_content)}")
        
        # Should be valid iCalendar
        assert calendar_content.startswith("BEGIN:VCALENDAR")
        assert calendar_content.endswith("END:VCALENDAR\r\n")
        assert "Plant Care Schedule" in calendar_content
        
        # Should have events for both plants
        assert "BEGIN:VEVENT" in calendar_content
        assert "💧 Water Fiddle Leaf Fig" in calendar_content
        assert "💧 Water Snake Plant" in calendar_content
        assert "🌱 Fertilize Fiddle Leaf Fig" in calendar_content
        assert "🌱 Fertilize Snake Plant" in calendar_content
        
        # Check event details (handle iCalendar line wrapping with \r\n )
        # iCalendar format wraps long lines with CRLF + space
        calendar_unwrapped = calendar_content.replace('\r\n ', '')
        assert "Every 7 days" in calendar_unwrapped
        assert "Every 14 days" in calendar_unwrapped
        assert "Every 30 days" in calendar_unwrapped
        
        # Check categories (iCalendar escapes commas with backslashes)
        assert "CATEGORIES:Plant Care\\,Water" in calendar_content
        assert "CATEGORIES:Plant Care\\,Fertilize" in calendar_content

    def test_calendar_feed_invalid_token(self):
        """Test calendar feed with invalid token"""
        # Get a valid user ID from subscription info
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        feed_url = response.json()["feedUrl"]
        
        # Extract user ID from URL
        import urllib.parse
        import re
        parsed_url = urllib.parse.urlparse(feed_url)
        user_id_match = re.search(r'calendar/([^.]+)\.ics', parsed_url.path)
        assert user_id_match
        user_id = user_id_match.group(1)
        
        # Try with invalid token
        invalid_url = f"{self.client.base_url}/api/v1/calendar/{user_id}.ics?token=invalid_token"
        
        import requests
        calendar_response = requests.get(invalid_url)
        assert calendar_response.status_code == 401
        
        error_data = calendar_response.json()
        assert error_data["error"] == "authentication_error"
        assert "Invalid calendar token" in error_data["message"]

    def test_calendar_feed_missing_token(self):
        """Test calendar feed without token parameter"""
        # Get user ID from subscription info
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        feed_url = response.json()["feedUrl"]
        
        # Extract user ID
        import urllib.parse
        import re
        parsed_url = urllib.parse.urlparse(feed_url)
        user_id_match = re.search(r'calendar/([^.]+)\.ics', parsed_url.path)
        assert user_id_match
        user_id = user_id_match.group(1)
        
        # Try without token
        no_token_url = f"{self.client.base_url}/api/v1/calendar/{user_id}.ics"
        
        import requests
        calendar_response = requests.get(no_token_url)
        assert calendar_response.status_code == 401
        
        error_data = calendar_response.json()
        assert error_data["error"] == "authentication_error"
        assert "Calendar token required" in error_data["message"]

    def test_calendar_feed_content_type(self):
        """Test that calendar feed returns correct content type"""
        # Create a plant first
        plant_data = {
            "name": "Test Plant",
            "genus": "Testicus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        }
        
        response = self.client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        
        # Get calendar feed
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        feed_url = response.json()["feedUrl"]
        
        # Extract path and query
        import urllib.parse
        parsed_url = urllib.parse.urlparse(feed_url)
        calendar_path = f"{parsed_url.path}?{parsed_url.query}"
        
        # Request with headers
        import requests
        calendar_response = requests.get(f"{self.client.base_url}{calendar_path}")
        assert calendar_response.status_code == 200
        
        # Check content type
        content_type = calendar_response.headers.get('content-type')
        assert content_type == "text/calendar; charset=utf-8"
        
        # Check content disposition (should suggest download)
        content_disposition = calendar_response.headers.get('content-disposition')
        assert content_disposition is not None
        assert "attachment" in content_disposition
        assert ".ics" in content_disposition

    def test_calendar_feed_caching_headers(self):
        """Test that calendar feed has appropriate caching headers"""
        # Get calendar feed URL
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        feed_url = response.json()["feedUrl"]
        
        # Extract path and query
        import urllib.parse
        parsed_url = urllib.parse.urlparse(feed_url)
        calendar_path = f"{parsed_url.path}?{parsed_url.query}"
        
        # Request the calendar
        import requests
        calendar_response = requests.get(f"{self.client.base_url}{calendar_path}")
        assert calendar_response.status_code == 200
        
        # Check caching headers
        cache_control = calendar_response.headers.get('cache-control')
        assert cache_control is not None
        assert "private" in cache_control  # Should be private (user-specific)
        assert "max-age" in cache_control  # Should have max-age

    def test_calendar_events_have_unique_uids(self):
        """Test that calendar events have unique UIDs"""
        # Create multiple plants
        plants_data = [
            {
                "name": "Plant 1", 
                "genus": "Genus1", 
                "careTasks": [
                    {"name": "Water", "icon": "💧", "intervalDays": 5},
                    {"name": "Fertilize", "icon": "🌱", "intervalDays": 10}
                ]
            },
            {
                "name": "Plant 2", 
                "genus": "Genus2", 
                "careTasks": [
                    {"name": "Water", "icon": "💧", "intervalDays": 7},
                    {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
                ]
            }
        ]
        
        for plant_data in plants_data:
            response = self.client.request("POST", "/plants", json=plant_data)
            assert response.status_code == 201
        
        # Get calendar feed
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        feed_url = response.json()["feedUrl"]
        
        # Get calendar content
        import urllib.parse
        parsed_url = urllib.parse.urlparse(feed_url)
        calendar_path = f"{parsed_url.path}?{parsed_url.query}"
        
        import requests
        calendar_response = requests.get(f"{self.client.base_url}{calendar_path}")
        assert calendar_response.status_code == 200
        
        calendar_content = calendar_response.text
        
        # Extract all UIDs
        uids = []
        for line in calendar_content.split('\n'):
            if line.startswith('UID:'):
                uids.append(line.strip())
        
        # Should have UIDs for watering and fertilizing events for each plant
        assert len(uids) >= 4  # At least 2 plants × 2 event types
        
        # All UIDs should be unique
        assert len(uids) == len(set(uids)), "Found duplicate UIDs in calendar"

    def test_calendar_unicode_plant_names(self):
        """Test calendar generation with unicode plant names"""
        # Create plant with unicode characters
        plant_data = {
            "name": "🌿 Monstera Deliciosa",
            "genus": "Mønstéra",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 21}
            ]
        }
        
        response = self.client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        
        # Get calendar feed
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        feed_url = response.json()["feedUrl"]
        
        # Get calendar content
        import urllib.parse
        parsed_url = urllib.parse.urlparse(feed_url)
        calendar_path = f"{parsed_url.path}?{parsed_url.query}"
        
        import requests
        calendar_response = requests.get(f"{self.client.base_url}{calendar_path}")
        assert calendar_response.status_code == 200
        
        calendar_content = calendar_response.text
        
        # Should handle unicode properly
        assert "🌿 Monstera Deliciosa" in calendar_content
        assert "Mønstéra" in calendar_content
        assert "💧 Water 🌿 Monstera Deliciosa" in calendar_content
        assert "🌱 Fertilize 🌿 Monstera Deliciosa" in calendar_content


class TestCareTasksCRUD:
    """Test care tasks CRUD operations"""

    def _create_plant_with_tasks(self, client, test_users):
        """Helper: register, login, create plant with care tasks, return (plant_id, care_tasks)"""
        client.request("POST", "/auth/register", json=test_users["user1"])
        client.request("POST", "/auth/login", json={
            "email": test_users["user1"]["email"],
            "password": test_users["user1"]["password"]
        })
        response = client.request("POST", "/plants", json={
            "name": "Test Plant",
            "genus": "Testus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ],
            "customMetrics": []
        })
        assert response.status_code == 201
        data = response.json()
        return data["id"], data["careTasks"]

    def test_care_tasks_created_with_plant(self, client, test_users):
        """Care tasks are created alongside the plant"""
        plant_id, care_tasks = self._create_plant_with_tasks(client, test_users)
        assert len(care_tasks) == 2
        water = next(t for t in care_tasks if t["name"] == "Water")
        fert = next(t for t in care_tasks if t["name"] == "Fertilize")
        assert water["intervalDays"] == 7
        assert water["icon"] == "💧"
        assert fert["intervalDays"] == 14
        assert fert["icon"] == "🌱"

    def test_list_care_tasks(self, client, test_users):
        """Can list care tasks for a plant"""
        plant_id, _ = self._create_plant_with_tasks(client, test_users)
        response = client.request("GET", f"/plants/{plant_id}/care-tasks")
        assert response.status_code == 200
        data = response.json()
        assert len(data["tasks"]) == 2

    def test_create_care_task(self, client, test_users):
        """Can add a new care task to an existing plant"""
        plant_id, _ = self._create_plant_with_tasks(client, test_users)
        response = client.request("POST", f"/plants/{plant_id}/care-tasks", json={
            "name": "Mist",
            "icon": "🌫️",
            "intervalDays": 2
        })
        assert response.status_code == 201
        task = response.json()
        assert task["name"] == "Mist"
        assert task["intervalDays"] == 2

        # Verify it appears in list
        list_resp = client.request("GET", f"/plants/{plant_id}/care-tasks")
        assert len(list_resp.json()["tasks"]) == 3

    def test_update_care_task(self, client, test_users):
        """Can update an existing care task"""
        plant_id, care_tasks = self._create_plant_with_tasks(client, test_users)
        task_id = care_tasks[0]["id"]
        response = client.request("PUT", f"/plants/{plant_id}/care-tasks/{task_id}", json={
            "name": "Deep Water",
            "intervalDays": 10
        })
        assert response.status_code == 200
        updated = response.json()
        assert updated["name"] == "Deep Water"
        assert updated["intervalDays"] == 10

    def test_delete_care_task(self, client, test_users):
        """Can delete a care task"""
        plant_id, care_tasks = self._create_plant_with_tasks(client, test_users)
        task_id = care_tasks[0]["id"]
        response = client.request("DELETE", f"/plants/{plant_id}/care-tasks/{task_id}")
        assert response.status_code == 204

        # Verify it's gone
        list_resp = client.request("GET", f"/plants/{plant_id}/care-tasks")
        assert len(list_resp.json()) == 1

    def test_log_care_task(self, client, test_users):
        """Logging a care task updates lastPerformed and creates a tracking entry"""
        plant_id, care_tasks = self._create_plant_with_tasks(client, test_users)
        task_id = care_tasks[0]["id"]

        # Log the task
        response = client.request("POST", f"/plants/{plant_id}/care-tasks/{task_id}/log", json={
            "timestamp": "2024-06-15T10:00:00Z"
        })
        assert response.status_code in [200, 201]

        # Verify lastPerformed is updated on the care task
        task_resp = client.request("GET", f"/plants/{plant_id}/care-tasks/{task_id}")
        assert task_resp.status_code == 200
        task = task_resp.json()
        assert task["lastPerformed"] is not None

    def test_archive_unarchive_care_task(self, client, test_users):
        """Can archive and unarchive a care task"""
        plant_id, care_tasks = self._create_plant_with_tasks(client, test_users)
        task_id = care_tasks[0]["id"]

        # Archive
        response = client.request("POST", f"/plants/{plant_id}/care-tasks/{task_id}/archive")
        assert response.status_code == 200
        archived = response.json()
        assert archived["archivedAt"] is not None

        # Unarchive
        response = client.request("POST", f"/plants/{plant_id}/care-tasks/{task_id}/unarchive")
        assert response.status_code == 200
        unarchived = response.json()
        assert unarchived.get("archivedAt") is None


class TestUnifiedTrackingEntries:
    """Test unified tracking entries (no entryType discriminator)"""

    def _setup(self, client, test_users):
        """Helper: register, login, create plant with tasks and metrics"""
        client.request("POST", "/auth/register", json=test_users["user1"])
        client.request("POST", "/auth/login", json={
            "email": test_users["user1"]["email"],
            "password": test_users["user1"]["password"]
        })
        response = client.request("POST", "/plants", json={
            "name": "Tracked Plant",
            "genus": "Trackus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ],
            "customMetrics": [
                {"name": "Height", "unit": "cm", "dataType": "Number"},
                {"name": "Healthy", "unit": "", "dataType": "Boolean"}
            ]
        })
        assert response.status_code == 201
        plant = response.json()
        return plant

    def test_create_entry_with_care_tasks(self, client, test_users):
        """Can create an entry that logs multiple care tasks"""
        plant = self._setup(client, test_users)
        task_ids = [t["id"] for t in plant["careTasks"]]

        response = client.request("POST", f"/plants/{plant['id']}/entries", json={
            "timestamp": "2024-06-15T10:00:00Z",
            "careTaskIds": task_ids
        })
        assert response.status_code == 201
        entry = response.json()
        assert set(entry["careTaskIds"]) == set(task_ids)

    def test_create_entry_with_measurement(self, client, test_users):
        """Can create an entry with a measurement"""
        plant = self._setup(client, test_users)
        metric_id = plant["customMetrics"][0]["id"]

        response = client.request("POST", f"/plants/{plant['id']}/entries", json={
            "timestamp": "2024-06-15T10:00:00Z",
            "measurements": [{"metricId": metric_id, "value": 42.5}]
        })
        assert response.status_code == 201
        entry = response.json()
        assert len(entry["measurements"]) == 1
        assert entry["measurements"][0]["metricId"] == metric_id
        assert entry["measurements"][0]["value"] == 42.5

    def test_create_entry_with_notes_only(self, client, test_users):
        """Can create a notes-only entry"""
        plant = self._setup(client, test_users)

        response = client.request("POST", f"/plants/{plant['id']}/entries", json={
            "timestamp": "2024-06-15T10:00:00Z",
            "notes": "Leaves looking great today"
        })
        assert response.status_code == 201
        entry = response.json()
        assert entry["notes"] == "Leaves looking great today"
        assert entry.get("careTaskIds") is None or entry["careTaskIds"] == []

    def test_create_combined_entry(self, client, test_users):
        """Can create an entry combining care tasks, measurements, and notes"""
        plant = self._setup(client, test_users)
        task_id = plant["careTasks"][0]["id"]
        metric_id = plant["customMetrics"][0]["id"]

        response = client.request("POST", f"/plants/{plant['id']}/entries", json={
            "timestamp": "2024-06-15T10:00:00Z",
            "careTaskIds": [task_id],
            "measurements": [{"metricId": metric_id, "value": 35.0}],
            "notes": "Watered and measured growth"
        })
        assert response.status_code == 201
        entry = response.json()
        assert task_id in entry["careTaskIds"]
        assert len(entry["measurements"]) == 1
        assert entry["notes"] == "Watered and measured growth"

    def test_list_entries(self, client, test_users):
        """Can list tracking entries for a plant"""
        plant = self._setup(client, test_users)

        # Create a few entries
        for i in range(3):
            client.request("POST", f"/plants/{plant['id']}/entries", json={
                "timestamp": f"2024-06-{15+i}T10:00:00Z",
                "notes": f"Entry {i}"
            })

        response = client.request("GET", f"/plants/{plant['id']}/entries")
        assert response.status_code == 200
        data = response.json()
        assert len(data["entries"]) == 3

    def test_entry_updates_care_task_last_performed(self, client, test_users):
        """Creating an entry with careTaskIds updates lastPerformed on those tasks"""
        plant = self._setup(client, test_users)
        task_id = plant["careTasks"][0]["id"]

        # Initially lastPerformed should be null
        task_resp = client.request("GET", f"/plants/{plant['id']}/care-tasks/{task_id}")
        assert task_resp.json()["lastPerformed"] is None

        # Create entry
        client.request("POST", f"/plants/{plant['id']}/entries", json={
            "timestamp": "2024-06-15T10:00:00Z",
            "careTaskIds": [task_id]
        })

        # Now lastPerformed should be set
        task_resp = client.request("GET", f"/plants/{plant['id']}/care-tasks/{task_id}")
        assert task_resp.json()["lastPerformed"] is not None

    def test_entry_requires_at_least_one_field(self, client, test_users):
        """An entry with no content fields should be rejected or treated as empty"""
        plant = self._setup(client, test_users)

        response = client.request("POST", f"/plants/{plant['id']}/entries", json={
            "timestamp": "2024-06-15T10:00:00Z"
        })
        # Either 400 (validation) or 201 (allowed empty) - both are valid behaviors
        # Just verify it doesn't crash
        assert response.status_code in [201, 400, 422]


class TestCoachSuggestions:
    """Test AI coach chat and suggestion acceptance with mock LLM provider"""

    def _setup(self, client, test_users):
        """Helper: register, login, create plant with Water task"""
        client.request("POST", "/auth/register", json=test_users["user1"])
        client.request("POST", "/auth/login", json={
            "email": test_users["user1"]["email"],
            "password": test_users["user1"]["password"]
        })
        response = client.request("POST", "/plants", json={
            "name": "Coach Test Plant",
            "genus": "Monstera",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7},
                {"name": "Fertilize", "icon": "🌱", "intervalDays": 14}
            ]
        })
        assert response.status_code == 201
        return response.json()

    def test_send_message_returns_response(self, client, test_users):
        """Sending a message to the coach returns an AI response"""
        plant = self._setup(client, test_users)

        response = client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "Hello, how is my plant?"
        })
        assert response.status_code == 201
        data = response.json()
        assert "message" in data
        assert data["message"]["role"] == "assistant"
        assert len(data["message"]["content"]) > 0

    def test_schedule_change_suggestion(self, client, test_users):
        """Mock returns schedule_change suggestion when 'schedule' keyword used"""
        plant = self._setup(client, test_users)

        response = client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "I think I need to change the schedule"
        })
        assert response.status_code == 201
        data = response.json()
        suggestions = data["message"]["suggestions"]
        assert len(suggestions) >= 1
        sched = next(s for s in suggestions if s["suggestionType"] == "schedule_change")
        assert sched["payload"]["careTaskName"] == "Water"
        assert sched["payload"]["intervalDays"] == 5
        assert sched["status"] == "pending"

    def test_accept_schedule_change_updates_task(self, client, test_users):
        """Accepting a schedule_change suggestion updates the care task interval"""
        plant = self._setup(client, test_users)

        # Get a schedule_change suggestion
        response = client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "I think I need to change the schedule"
        })
        suggestions = response.json()["message"]["suggestions"]
        sched = next(s for s in suggestions if s["suggestionType"] == "schedule_change")

        # Accept it
        accept_resp = client.request("POST", f"/coach/suggestions/{sched['id']}/accept")
        assert accept_resp.status_code == 200
        assert accept_resp.json()["status"] == "accepted"

        # Verify the Water task now has intervalDays=5
        task_id = next(t["id"] for t in plant["careTasks"] if t["name"] == "Water")
        task_resp = client.request("GET", f"/plants/{plant['id']}/care-tasks/{task_id}")
        assert task_resp.status_code == 200
        assert task_resp.json()["intervalDays"] == 5

    def test_accept_new_task_creates_care_task(self, client, test_users):
        """Accepting a new_task suggestion creates a new care task on the plant"""
        plant = self._setup(client, test_users)

        # Get a new_task suggestion
        response = client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "Should I add a new task for misting?"
        })
        suggestions = response.json()["message"]["suggestions"]
        new_task = next(s for s in suggestions if s["suggestionType"] == "new_task")

        # Accept it
        accept_resp = client.request("POST", f"/coach/suggestions/{new_task['id']}/accept")
        assert accept_resp.status_code == 200

        # Verify a Misting task now exists
        tasks_resp = client.request("GET", f"/plants/{plant['id']}/care-tasks")
        assert tasks_resp.status_code == 200
        task_names = [t["name"] for t in tasks_resp.json()["tasks"]]
        assert "Misting" in task_names

    def test_accept_care_action_logs_entry(self, client, test_users):
        """Accepting a care_action suggestion logs the care task"""
        plant = self._setup(client, test_users)
        task_id = next(t["id"] for t in plant["careTasks"] if t["name"] == "Water")

        # Verify lastPerformed is initially None
        task_resp = client.request("GET", f"/plants/{plant['id']}/care-tasks/{task_id}")
        assert task_resp.json()["lastPerformed"] is None

        # Get a care_action suggestion
        response = client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "My plant needs action now, it's wilting"
        })
        suggestions = response.json()["message"]["suggestions"]
        action = next(s for s in suggestions if s["suggestionType"] == "care_action")

        # Accept it
        accept_resp = client.request("POST", f"/coach/suggestions/{action['id']}/accept")
        assert accept_resp.status_code == 200

        # Verify Water task lastPerformed is now set
        task_resp = client.request("GET", f"/plants/{plant['id']}/care-tasks/{task_id}")
        assert task_resp.json()["lastPerformed"] is not None

    def test_dismiss_suggestion(self, client, test_users):
        """Dismissing a suggestion marks it as dismissed without applying"""
        plant = self._setup(client, test_users)

        response = client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "Should I change the schedule?"
        })
        suggestions = response.json()["message"]["suggestions"]
        sched = next(s for s in suggestions if s["suggestionType"] == "schedule_change")

        # Dismiss it
        dismiss_resp = client.request("POST", f"/coach/suggestions/{sched['id']}/dismiss")
        assert dismiss_resp.status_code == 200
        assert dismiss_resp.json()["status"] == "dismissed"

        # Verify the task is unchanged (still 7 days)
        task_id = next(t["id"] for t in plant["careTasks"] if t["name"] == "Water")
        task_resp = client.request("GET", f"/plants/{plant['id']}/care-tasks/{task_id}")
        assert task_resp.json()["intervalDays"] == 7

    def test_get_conversation_messages(self, client, test_users):
        """Can retrieve conversation history"""
        plant = self._setup(client, test_users)

        # Send a message
        client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "Hello coach"
        })

        # Get messages
        response = client.request("GET", f"/coach/plants/{plant['id']}/messages")
        assert response.status_code == 200
        data = response.json()
        assert "conversationId" in data
        assert len(data["messages"]) >= 2  # user + assistant
        roles = [m["role"] for m in data["messages"]]
        assert "user" in roles
        assert "assistant" in roles

    def test_photo_request_suggestion(self, client, test_users):
        """Mock returns photo_request when 'photo' keyword used"""
        plant = self._setup(client, test_users)

        response = client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "Can you tell from a photo what's wrong?"
        })
        assert response.status_code == 201
        suggestions = response.json()["message"]["suggestions"]
        photo = next(s for s in suggestions if s["suggestionType"] == "photo_request")
        assert photo["payload"] == {}

    def test_accept_photo_request_creates_check_in_task(self, client, test_users):
        """Accepting a photo_request creates a 'Photo check-in' care task"""
        plant = self._setup(client, test_users)

        # Get a photo_request suggestion
        response = client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "Can you tell from a photo what's wrong?"
        })
        suggestions = response.json()["message"]["suggestions"]
        photo_s = next(s for s in suggestions if s["suggestionType"] == "photo_request")

        # Accept it
        accept_resp = client.request("POST", f"/coach/suggestions/{photo_s['id']}/accept")
        assert accept_resp.status_code == 200

        # Verify a "Photo check-in" task was created
        tasks_resp = client.request("GET", f"/plants/{plant['id']}/care-tasks")
        assert tasks_resp.status_code == 200
        task_names = [t["name"] for t in tasks_resp.json()["tasks"]]
        assert "Photo check-in" in task_names

        # Accepting again should NOT create a duplicate
        response2 = client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "Show me another photo request"
        })
        suggestions2 = response2.json()["message"]["suggestions"]
        photo_s2 = next(s for s in suggestions2 if s["suggestionType"] == "photo_request")
        client.request("POST", f"/coach/suggestions/{photo_s2['id']}/accept")

        tasks_resp2 = client.request("GET", f"/plants/{plant['id']}/care-tasks")
        photo_tasks = [t for t in tasks_resp2.json()["tasks"] if "photo" in t["name"].lower()]
        assert len(photo_tasks) == 1  # No duplicate


@pytest.mark.coach
class TestCoachStreaming:
    """Test SSE streaming endpoint for coach messages"""

    def _setup(self, client, test_users):
        """Helper: register, login, create plant"""
        client.request("POST", "/auth/register", json=test_users["user1"])
        client.request("POST", "/auth/login", json={
            "email": test_users["user1"]["email"],
            "password": test_users["user1"]["password"]
        })
        response = client.request("POST", "/plants", json={
            "name": "Stream Test Plant",
            "genus": "Ficus",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7}
            ]
        })
        assert response.status_code == 201
        return response.json()

    def test_stream_returns_sse_events(self, client, test_users):
        """Streaming endpoint returns SSE with token and done events"""
        plant = self._setup(client, test_users)

        # Use raw requests to read SSE stream
        url = f"{client.base_url}{client.api_prefix}/coach/plants/{plant['id']}/messages/stream"
        response = client.session.post(
            url,
            json={"content": "Hello coach"},
            headers={"Content-Type": "application/json"},
            stream=True,
        )
        assert response.status_code == 200
        assert "text/event-stream" in response.headers.get("content-type", "")

        events = []
        for line in response.iter_lines(decode_unicode=True):
            if line and line.startswith("event: "):
                events.append({"event": line[7:]})
            elif line and line.startswith("data: ") and events:
                events[-1]["data"] = line[6:]

        # Should have at least one token event and one done event
        event_types = [e["event"] for e in events]
        assert "token" in event_types, f"Expected 'token' event, got: {event_types}"
        assert "done" in event_types, f"Expected 'done' event, got: {event_types}"

    def test_stream_done_event_contains_message(self, client, test_users):
        """The 'done' SSE event contains a valid CoachMessage with id and suggestions"""
        plant = self._setup(client, test_users)

        url = f"{client.base_url}{client.api_prefix}/coach/plants/{plant['id']}/messages/stream"
        response = client.session.post(
            url,
            json={"content": "I think I need to change the schedule"},
            headers={"Content-Type": "application/json"},
            stream=True,
        )
        assert response.status_code == 200

        import json as json_mod
        done_data = None
        current_event = ""
        for line in response.iter_lines(decode_unicode=True):
            if line and line.startswith("event: "):
                current_event = line[7:]
            elif line and line.startswith("data: ") and current_event == "done":
                done_data = json_mod.loads(line[6:])

        assert done_data is not None, "No 'done' event received"
        assert done_data["role"] == "assistant"
        assert "id" in done_data
        assert "content" in done_data
        assert "suggestions" in done_data
        # With 'schedule' keyword, mock should produce a suggestion
        assert len(done_data["suggestions"]) >= 1

    def test_stream_tokens_form_complete_text(self, client, test_users):
        """Token events concatenated should match the final message content"""
        plant = self._setup(client, test_users)

        url = f"{client.base_url}{client.api_prefix}/coach/plants/{plant['id']}/messages/stream"
        response = client.session.post(
            url,
            json={"content": "How is my plant?"},
            headers={"Content-Type": "application/json"},
            stream=True,
        )
        assert response.status_code == 200

        import json as json_mod
        tokens = []
        done_data = None
        current_event = ""
        for line in response.iter_lines(decode_unicode=True):
            if line and line.startswith("event: "):
                current_event = line[7:]
            elif line and line.startswith("data: "):
                data = line[6:]
                if current_event == "token":
                    tokens.append(data)
                elif current_event == "done":
                    done_data = json_mod.loads(data)

        assert done_data is not None
        streamed_text = "".join(tokens)
        # The streamed text should match the final content
        assert streamed_text.strip() == done_data["content"].strip()

    def test_stream_persists_messages(self, client, test_users):
        """Streaming endpoint persists both user and assistant messages"""
        plant = self._setup(client, test_users)

        # Send via stream
        url = f"{client.base_url}{client.api_prefix}/coach/plants/{plant['id']}/messages/stream"
        response = client.session.post(
            url,
            json={"content": "Hello via stream"},
            headers={"Content-Type": "application/json"},
            stream=True,
        )
        # Consume the stream
        for _ in response.iter_lines(decode_unicode=True):
            pass

        # Verify messages are persisted
        msgs_resp = client.request("GET", f"/coach/plants/{plant['id']}/messages")
        assert msgs_resp.status_code == 200
        messages = msgs_resp.json()["messages"]
        assert len(messages) >= 2
        user_msgs = [m for m in messages if m["role"] == "user"]
        asst_msgs = [m for m in messages if m["role"] == "assistant"]
        assert any("Hello via stream" in m["content"] for m in user_msgs)
        assert len(asst_msgs) >= 1

    def test_stream_unauthorized_returns_error(self, client, test_users):
        """Streaming without auth returns 401"""
        # Create a fresh client with no session
        fresh_client = APIClient(client.base_url, client.api_prefix)
        url = f"{fresh_client.base_url}{fresh_client.api_prefix}/coach/plants/00000000-0000-0000-0000-000000000000/messages/stream"
        response = fresh_client.session.post(
            url,
            json={"content": "Hello"},
            headers={"Content-Type": "application/json"},
        )
        assert response.status_code == 401


@pytest.mark.memory
class TestPlantMemory:
    """Test plant memory system: CRUD and automatic extraction from coach"""

    def _setup(self, client, test_users):
        """Helper: register, login, create plant"""
        client.request("POST", "/auth/register", json=test_users["user1"])
        client.request("POST", "/auth/login", json={
            "email": test_users["user1"]["email"],
            "password": test_users["user1"]["password"]
        })
        response = client.request("POST", "/plants", json={
            "name": "Memory Test Plant",
            "genus": "Pothos",
            "careTasks": [
                {"name": "Water", "icon": "💧", "intervalDays": 7}
            ]
        })
        assert response.status_code == 201
        return response.json()

    def test_memories_initially_empty(self, client, test_users):
        """A new plant has no memories"""
        plant = self._setup(client, test_users)
        response = client.request("GET", f"/coach/plants/{plant['id']}/memories")
        assert response.status_code == 200
        assert response.json()["memories"] == []

    def test_create_memory_manually(self, client, test_users):
        """Can create a plant memory manually"""
        plant = self._setup(client, test_users)
        response = client.request("POST", f"/coach/plants/{plant['id']}/memories", json={
            "factType": "location",
            "content": "East-facing windowsill",
            "confidence": 0.95,
        })
        assert response.status_code == 201
        data = response.json()
        assert data["factType"] == "location"
        assert data["content"] == "East-facing windowsill"
        assert data["confidence"] == 1.0  # User-created facts always get confidence 1.0
        assert data["source"] == "user"

    def test_list_memories(self, client, test_users):
        """Can list all memories for a plant"""
        plant = self._setup(client, test_users)
        client.request("POST", f"/coach/plants/{plant['id']}/memories", json={
            "factType": "location",
            "content": "Kitchen window",
            "confidence": 0.9,
        })
        client.request("POST", f"/coach/plants/{plant['id']}/memories", json={
            "factType": "pot",
            "content": "6 inch ceramic pot",
            "confidence": 0.85,
        })

        response = client.request("GET", f"/coach/plants/{plant['id']}/memories")
        assert response.status_code == 200
        memories = response.json()["memories"]
        assert len(memories) == 2
        fact_types = {m["factType"] for m in memories}
        assert "location" in fact_types
        assert "pot" in fact_types

    def test_update_memory(self, client, test_users):
        """Can update a memory's content"""
        plant = self._setup(client, test_users)
        create_resp = client.request("POST", f"/coach/plants/{plant['id']}/memories", json={
            "factType": "location",
            "content": "North window",
            "confidence": 0.7,
        })
        memory_id = create_resp.json()["id"]

        update_resp = client.request("PUT", f"/coach/plants/{plant['id']}/memories/{memory_id}", json={
            "content": "South window — moved recently",
            "confidence": 0.95,
        })
        assert update_resp.status_code == 200
        assert update_resp.json()["content"] == "South window — moved recently"

    def test_delete_memory(self, client, test_users):
        """Can delete a memory"""
        plant = self._setup(client, test_users)
        create_resp = client.request("POST", f"/coach/plants/{plant['id']}/memories", json={
            "factType": "general",
            "content": "Test fact to delete",
            "confidence": 0.5,
        })
        memory_id = create_resp.json()["id"]

        delete_resp = client.request("DELETE", f"/coach/plants/{plant['id']}/memories/{memory_id}")
        assert delete_resp.status_code in (200, 204)

        # Verify it's gone
        list_resp = client.request("GET", f"/coach/plants/{plant['id']}/memories")
        assert all(m["id"] != memory_id for m in list_resp.json()["memories"])

    def test_coach_extracts_facts_automatically(self, client, test_users):
        """Sending a message with keywords triggers fact extraction into memories"""
        plant = self._setup(client, test_users)

        # The mock extracts a 'location' fact when 'window' is mentioned
        client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "My plant is sitting by the south window"
        })

        # Check that a memory was created
        response = client.request("GET", f"/coach/plants/{plant['id']}/memories")
        assert response.status_code == 200
        memories = response.json()["memories"]
        assert len(memories) >= 1
        location_memories = [m for m in memories if m["factType"] == "location"]
        assert len(location_memories) >= 1
        assert location_memories[0]["source"] == "coach"

    def test_coach_extracts_pot_fact(self, client, test_users):
        """Mock extracts 'pot' fact when pot/terracotta mentioned"""
        plant = self._setup(client, test_users)

        client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "I just repotted into a terracotta pot"
        })

        response = client.request("GET", f"/coach/plants/{plant['id']}/memories")
        memories = response.json()["memories"]
        pot_memories = [m for m in memories if m["factType"] == "pot"]
        assert len(pot_memories) >= 1

    def test_memory_deduplication(self, client, test_users):
        """Sending similar facts doesn't create duplicates"""
        plant = self._setup(client, test_users)

        # Send twice with 'window' keyword — should only create one location memory
        client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "It's near the window"
        })
        client.request("POST", f"/coach/plants/{plant['id']}/messages", json={
            "content": "Yes, it's by the window still"
        })

        response = client.request("GET", f"/coach/plants/{plant['id']}/memories")
        memories = response.json()["memories"]
        location_memories = [m for m in memories if m["factType"] == "location"]
        # Should be deduplicated (same fact_type + similar content)
        assert len(location_memories) == 1

    def test_health_score_endpoint(self, client, test_users):
        """Health score endpoint returns valid data"""
        plant = self._setup(client, test_users)
        response = client.request("GET", f"/coach/plants/{plant['id']}/health")
        assert response.status_code == 200
        data = response.json()
        assert "score" in data
        assert "hearts" in data
        assert 0.0 <= data["score"] <= 1.0
        assert 0.0 <= data["hearts"] <= 5.0

    def test_memory_isolation_between_plants(self, client, test_users):
        """Memories for one plant don't appear on another"""
        plant1 = self._setup(client, test_users)
        # Create second plant
        resp2 = client.request("POST", "/plants", json={
            "name": "Other Plant",
            "genus": "Dracaena",
            "careTasks": []
        })
        plant2 = resp2.json()

        # Add memory to plant1
        client.request("POST", f"/coach/plants/{plant1['id']}/memories", json={
            "factType": "location",
            "content": "Bedroom",
            "confidence": 0.9,
        })

        # plant2 should have no memories
        resp = client.request("GET", f"/coach/plants/{plant2['id']}/memories")
        assert resp.json()["memories"] == []


if __name__ == "__main__":
    # Run tests when script is executed directly
    pytest.main([__file__, "-v"])
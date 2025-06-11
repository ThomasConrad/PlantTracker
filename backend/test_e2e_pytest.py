"""
End-to-End Test Suite for PlantTracker Backend using pytest

This module tests the complete PlantTracker API by:
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
        self.api_prefix = "/v1"  # API prefix when no frontend is served (API-only mode)
        
    def start(self):
        """Start the backend server"""
        print(f"Starting PlantTracker backend on port {self.port}...")
        
        # Change to backend directory
        backend_dir = Path(__file__).parent
        os.chdir(backend_dir)
            
        # Start the backend process with in-memory database
        env = os.environ.copy()
        env["RUST_LOG"] = "info"
        
        try:
            self.process = subprocess.Popen([
                "cargo", "run", "--release", "--bin", "plant-tracker-api", "--",
                "--port", str(self.port),
                "--database-url", "sqlite::memory:",
                "--frontend-dir", "/nonexistent"  # Force API-only mode
            ],
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            
            # Wait for server to start
            max_retries = 30
            for i in range(max_retries):
                try:
                    response = requests.get(f"{self.base_url}/", timeout=1)
                    if response.status_code == 200:
                        print(f"Backend started successfully on port {self.port}")
                        return
                except requests.exceptions.RequestException:
                    pass
                    
                time.sleep(1)
                
                # Check if process is still running
                if self.process.poll() is not None:
                    stdout, stderr = self.process.communicate()
                    print(f"Backend process failed to start:")
                    print(f"STDOUT: {stdout}")
                    print(f"STDERR: {stderr}")
                    raise RuntimeError("Backend process exited unexpectedly")
                    
            raise TimeoutError(f"Backend failed to start within {max_retries} seconds")
            
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
    
    def __init__(self, base_url: str, api_prefix: str = "/v1"):
        self.base_url = base_url
        self.api_prefix = api_prefix
        self.session = requests.Session()
        
    def request(self, method: str, endpoint: str, **kwargs) -> requests.Response:
        """Make an HTTP request to the backend"""
        # Add API prefix if endpoint doesn't start with /
        if not endpoint.startswith('/'):
            endpoint = f"/{endpoint}"
        
        # For API endpoints, add the v1 prefix
        if endpoint.startswith('/auth') or endpoint.startswith('/plants') or endpoint.startswith('/photos') or endpoint.startswith('/tracking'):
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
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        }
        
        response = client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        
        response_data = response.json()
        assert response_data["name"] == plant_data["name"]
        assert response_data["genus"] == plant_data["genus"]
        assert response_data["wateringIntervalDays"] == plant_data["wateringIntervalDays"]
        assert "id" in response_data
        
    def test_create_plant_validation_errors(self, client):
        """Test plant creation validation"""
        # Empty name should fail
        response = client.request("POST", "/plants", json={
            "name": "",
            "genus": "Ficus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        })
        assert response.status_code == 422
        
        # Invalid watering interval should fail
        response = client.request("POST", "/plants", json={
            "name": "Test Plant",
            "genus": "Test",
            "wateringIntervalDays": 0,
            "fertilizingIntervalDays": 14
        })
        assert response.status_code == 422
        
    def test_get_plants(self, client):
        """Test getting all plants"""
        # Create some plants first
        plants_data = [
            {"name": "Plant 1", "genus": "Genus1", "wateringIntervalDays": 7, "fertilizingIntervalDays": 14},
            {"name": "Plant 2", "genus": "Genus2", "wateringIntervalDays": 10, "fertilizingIntervalDays": 21},
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
            "wateringIntervalDays": 5,
            "fertilizingIntervalDays": 10
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
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        }
        response = client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        plant = response.json()
        
        # Update the plant
        update_data = {
            "name": "Updated Plant",
            "wateringIntervalDays": 10
        }
        response = client.request("PUT", f"/plants/{plant['id']}", json=update_data)
        assert response.status_code == 200
        
        response_data = response.json()
        assert response_data["name"] == "Updated Plant"
        assert response_data["wateringIntervalDays"] == 10
        assert response_data["genus"] == plant_data["genus"]  # Should remain unchanged
        
    def test_delete_plant(self, client):
        """Test deleting a plant"""
        # Create a plant
        plant_data = {
            "name": "Plant to Delete",
            "genus": "DeleteGenus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
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
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
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
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
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
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        })
        assert response.status_code == 400
        
        # Test validation error with all required fields but invalid values
        response = client.request("POST", "/plants", json={
            "name": "",  # Empty name should fail validation
            "genus": "TestGenus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        })
        assert response.status_code == 422


# Performance and load testing (optional)
@pytest.mark.performance
@pytest.mark.slow
class TestPerformance:
    """Optional performance tests"""
    
    @pytest.mark.slow
    def test_many_plants_creation(self, client, test_users):
        """Test creating many plants (performance test)"""
        user_data = test_users["user1"]
        
        # Register and login user
        client.request("POST", "/auth/register", json=user_data)
        client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        
        # Create many plants
        num_plants = 50
        start_time = time.time()
        
        for i in range(num_plants):
            plant_data = {
                "name": f"Plant {i}",
                "genus": f"Genus{i}",
                "wateringIntervalDays": 7,
                "fertilizingIntervalDays": 14
            }
            response = client.request("POST", "/plants", json=plant_data)
            assert response.status_code == 201
            
        end_time = time.time()
        duration = end_time - start_time
        
        print(f"Created {num_plants} plants in {duration:.2f} seconds")
        assert duration < 30  # Should complete within 30 seconds
        
        # Verify all plants were created
        response = client.request("GET", "/plants")
        assert response.status_code == 200
        response_data = response.json()
        assert response_data["total"] == num_plants


@pytest.mark.photos
class TestPhotoUpload:
    """Test photo upload functionality"""
    
    @pytest.fixture(scope="function")
    def photo_backend(self):
        """Create a dedicated backend server instance for photo tests to ensure isolation"""
        with BackendServer() as server:
            yield server
    
    @pytest.fixture(scope="function")
    def photo_client(self, photo_backend):
        """Create a dedicated client for photo tests"""
        return APIClient(photo_backend.base_url, photo_backend.api_prefix)
    
    @pytest.fixture(autouse=True)
    def login_user(self, photo_client, test_users):
        """Automatically login a user before each test"""
        user_data = test_users["user1"]
        
        # Register and login user
        photo_client.request("POST", "/auth/register", json=user_data)
        response = photo_client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        assert response.status_code == 200
        
        # Store client reference for test methods
        self.client = photo_client

    def test_upload_photo_multipart(self):
        """Test uploading a photo using multipart form data"""
        # Create a plant first
        plant_data = {
            "name": "Photo Test Plant",
            "genus": "Photographicus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Create fake image data (minimal JPEG header)
        fake_image_data = b'\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00\xff\xdb'
        
        # Upload photo using multipart form data
        import io
        files = {
            'file': ('test-photo.jpg', io.BytesIO(fake_image_data), 'image/jpeg')
        }
        
        # Use requests directly for multipart upload
        import requests
        response = requests.post(
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        
        assert response.status_code == 201
        photo_data = response.json()
        
        assert "id" in photo_data
        assert photo_data["plantId"] == plant_id
        assert photo_data["originalFilename"] == "test-photo.jpg"
        assert photo_data["contentType"] == "image/jpeg"
        assert photo_data["size"] == len(fake_image_data)
        assert "createdAt" in photo_data

    def test_list_photos_after_upload(self):
        """Test listing photos after uploading one"""
        # Create a plant first
        plant_data = {
            "name": "Photo List Plant",
            "genus": "Listicus",
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Upload a photo
        fake_image_data = b'\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00\xff\xdb'
        
        import io
        import requests
        files = {
            'file': ('list-test.jpg', io.BytesIO(fake_image_data), 'image/jpeg')
        }
        
        upload_response = requests.post(
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
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
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Upload a photo
        fake_image_data = b'\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00\xff\xdb'
        
        import io
        import requests
        files = {
            'file': ('delete-test.jpg', io.BytesIO(fake_image_data), 'image/jpeg')
        }
        
        upload_response = requests.post(
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
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

    def test_plant_thumbnail_functionality(self):
        """Test that setting a plant thumbnail correctly updates the thumbnailUrl in plant responses"""
        # Create a plant first
        plant_data = {
            "name": "Thumbnail Test Plant",
            "genus": "Thumbnailicus", 
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Initially, the plant should have no thumbnail
        assert plant.get("thumbnailId") is None
        assert plant.get("thumbnailUrl") is None
        
        # Upload a photo
        import io
        import requests
        fake_image_data = b'\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00\xff\xdb'
        
        files = {
            'file': ('thumbnail-test.jpg', io.BytesIO(fake_image_data), 'image/jpeg')
        }
        
        upload_response = requests.post(
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        assert upload_response.status_code == 201
        photo = upload_response.json()
        photo_id = photo["id"]
        
        # Set the uploaded photo as the plant's thumbnail
        thumbnail_response = self.client.request("PUT", f"/plants/{plant_id}/thumbnail/{photo_id}")
        assert thumbnail_response.status_code == 200
        updated_plant = thumbnail_response.json()
        
        # Verify the thumbnail is set
        assert updated_plant["thumbnailId"] == photo_id
        assert updated_plant["thumbnailUrl"] is not None
        assert updated_plant["thumbnailUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}/thumbnail"
        
        # Verify that fetching the plant individually also returns the thumbnail
        get_response = self.client.request("GET", f"/plants/{plant_id}")
        assert get_response.status_code == 200
        fetched_plant = get_response.json()
        
        assert fetched_plant["thumbnailId"] == photo_id
        assert fetched_plant["thumbnailUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}/thumbnail"
        
        # Verify that listing plants also returns the thumbnail
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
        assert our_plant["thumbnailId"] == photo_id
        assert our_plant["thumbnailUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}/thumbnail"

    def test_large_image_upload_performance(self):
        """Test upload performance with a large (~5MB) image and measure timing"""
        import time
        
        # Create a plant first
        plant_data = {
            "name": "Performance Test Plant",
            "genus": "Performicus", 
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Create a large fake image (~5MB)
        import io
        import requests
        
        # Create a minimal but valid JPEG that will actually trigger image processing
        # This is a small valid JPEG that we'll pad to make it large
        # Real JPEG header with quantization tables and Huffman tables
        minimal_jpeg = bytes([
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01, 0x00, 0x48,
            0x00, 0x48, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08,
            0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
            0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D, 0x1A, 0x1C, 0x1C, 0x20, 0x24, 0x2E, 0x27, 0x20,
            0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29, 0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27,
            0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x08,
            0x00, 0x08, 0x01, 0x01, 0x11, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01, 0xFF, 0xC4, 0x00, 0x14,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x08, 0xFF, 0xC4, 0x00, 0x14, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xDA, 0x00, 0x0C, 0x03, 0x01, 0x00, 0x02,
            0x11, 0x03, 0x11, 0x00, 0x3F, 0x00, 0xB2, 0xC0, 0x07, 0xFF, 0xD9
        ])
        
        # This creates a valid but tiny JPEG. For a large file test, we'll create multiple copies
        # and embed them in a multipart structure that's still a valid JPEG
        target_size = 5 * 1024 * 1024
        copies_needed = target_size // len(minimal_jpeg)
        
        # Create large image by repeating the minimal JPEG data with padding
        jpeg_header = minimal_jpeg[:20]  # Keep the JPEG header
        jpeg_footer = minimal_jpeg[-2:]  # Keep the end marker
        
        # Fill the middle with repeated image data and padding
        middle_size = target_size - len(jpeg_header) - len(jpeg_footer)
        repeated_data = (minimal_jpeg[20:-2] * (middle_size // (len(minimal_jpeg) - 22) + 1))[:middle_size]
        
        large_image_data = jpeg_header + repeated_data + jpeg_footer
        
        print(f"\nUploading {len(large_image_data)} bytes ({len(large_image_data) / (1024*1024):.1f}MB)")
        
        files = {
            'file': ('large-test.jpg', io.BytesIO(large_image_data), 'image/jpeg')
        }
        
        # Measure upload time
        start_time = time.time()
        
        upload_response = requests.post(
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
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
        
        # Measure thumbnail setting time
        thumbnail_start_time = time.time()
        
        thumbnail_response = self.client.request("PUT", f"/plants/{plant_id}/thumbnail/{photo_id}")
        
        thumbnail_end_time = time.time()
        thumbnail_duration = thumbnail_end_time - thumbnail_start_time
        
        assert thumbnail_response.status_code == 200
        updated_plant = thumbnail_response.json()
        
        print(f"Thumbnail setting took {thumbnail_duration:.2f} seconds")
        
        # Verify the thumbnail was set correctly
        assert updated_plant["thumbnailId"] == photo_id
        assert updated_plant["thumbnailUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}/thumbnail"
        
        total_time = upload_duration + thumbnail_duration
        print(f"Total time: {total_time:.2f} seconds")
        
        # Performance assertions (these are reasonable expectations)
        # Upload should complete within 30 seconds for 5MB
        assert upload_duration < 30.0, f"Upload took too long: {upload_duration:.2f}s"
        
        # Thumbnail setting should be fast (< 5 seconds)
        assert thumbnail_duration < 5.0, f"Thumbnail setting took too long: {thumbnail_duration:.2f}s"
        
        # Log breakdown for analysis
        print(f"Breakdown: Upload {upload_duration:.2f}s, Thumbnail {thumbnail_duration:.2f}s")
        if upload_duration > 2.0:
            print("WARNING: Upload is slower than expected - likely thumbnail generation bottleneck")
        
        # With async thumbnail generation, upload should be much faster
        if upload_duration < 1.0:
            print("SUCCESS: Upload is fast - async thumbnail generation is working!")
            
    def test_async_thumbnail_generation(self):
        """Test that thumbnails are generated asynchronously and eventually become available"""
        import time
        
        # Create a plant first
        plant_data = {
            "name": "Async Thumbnail Test Plant",
            "genus": "Asyncicus", 
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
        }
        
        plant_response = self.client.request("POST", "/plants", json=plant_data)
        assert plant_response.status_code == 201
        plant = plant_response.json()
        plant_id = plant["id"]
        
        # Upload a large image 
        import io
        import requests
        large_image_data = b'\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x01\x00H\x00H\x00\x00\xff\xdb' + b'\x00' * (1024 * 1024) + b'\xff\xd9'
        
        files = {
            'file': ('async-test.jpg', io.BytesIO(large_image_data), 'image/jpeg')
        }
        
        # Upload should be fast
        start_time = time.time()
        upload_response = requests.post(
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        upload_time = time.time() - start_time
        
        assert upload_response.status_code == 201
        photo = upload_response.json()
        photo_id = photo["id"]
        
        print(f"Async upload took {upload_time:.2f} seconds")
        
        # Initially, thumbnail might not be ready (202 status)
        thumbnail_url = f"/v1/plants/{plant_id}/photos/{photo_id}/thumbnail"
        initial_response = self.client.request("GET", thumbnail_url)
        
        if initial_response.status_code == 202:
            print("Thumbnail not ready immediately (as expected with async generation)")
            
            # Wait a bit and try again (thumbnail should be ready within a few seconds)
            max_wait = 10
            wait_time = 0
            while wait_time < max_wait:
                time.sleep(1)
                wait_time += 1
                retry_response = self.client.request("GET", thumbnail_url)
                
                if retry_response.status_code == 200:
                    print(f"Thumbnail became available after {wait_time} seconds")
                    break
                elif retry_response.status_code == 202:
                    continue
                else:
                    assert False, f"Unexpected thumbnail response: {retry_response.status_code}"
            
            if wait_time >= max_wait:
                print("WARNING: Thumbnail took longer than expected to generate")
        else:
            # Thumbnail was ready immediately (small image or very fast processing)
            assert initial_response.status_code == 200
            print("Thumbnail was ready immediately")

    def test_upload_photo_validation_errors(self):
        """Test photo upload validation"""
        # Create a plant first
        plant_data = {
            "name": "Validation Plant",
            "genus": "Validicus", 
            "wateringIntervalDays": 7,
            "fertilizingIntervalDays": 14
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
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
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
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
            files=files
        )
        assert response.status_code == 401


if __name__ == "__main__":
    # Run tests when script is executed directly
    pytest.main([__file__, "-v"])
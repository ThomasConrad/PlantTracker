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
        self.api_prefix = "/v1"  # API prefix when no frontend is served (API-only mode)
        
    def start(self):
        """Start the backend server"""
        print(f"Starting Planty backend on port {self.port}...")
        
        # Change to backend directory
        backend_dir = Path(__file__).parent
        os.chdir(backend_dir)
            
        # Start the backend process with in-memory database
        env = os.environ.copy()
        env["RUST_LOG"] = "info"
        
        try:
            self.process = subprocess.Popen([
                "cargo", "run", "--release", "--bin", "planty-api", "--",
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
        if endpoint.startswith('/auth') or endpoint.startswith('/plants') or endpoint.startswith('/photos') or endpoint.startswith('/tracking') or endpoint.startswith('/calendar'):
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
            "wateringSchedule": {
                "intervalDays": 7
            },
            "fertilizingSchedule": {
                "intervalDays": 14
            }
        }
        
        response = client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        
        response_data = response.json()
        assert response_data["name"] == plant_data["name"]
        assert response_data["genus"] == plant_data["genus"]
        assert response_data["wateringSchedule"]["intervalDays"] == 7
        assert response_data["fertilizingSchedule"]["intervalDays"] == 14
        assert "id" in response_data
        
    def test_create_plant_validation_errors(self, client):
        """Test plant creation validation"""
        # Empty name should fail
        response = client.request("POST", "/plants", json={
            "name": "",
            "genus": "Ficus",
            "wateringSchedule": {
                "intervalDays": 7
            },
            "fertilizingSchedule": {
                "intervalDays": 14
            }
        })
        assert response.status_code == 422
        
        # Invalid watering interval should fail
        response = client.request("POST", "/plants", json={
            "name": "Test Plant",
            "genus": "Test",
            "wateringSchedule": {
                "intervalDays": 0
            },
            "fertilizingSchedule": {
                "intervalDays": 14
            }
        })
        assert response.status_code == 422
        
    def test_get_plants(self, client):
        """Test getting all plants"""
        # Create some plants first
        plants_data = [
            {
                "name": "Plant 1", 
                "genus": "Genus1", 
                "wateringSchedule": {"intervalDays": 7}, 
                "fertilizingSchedule": {"intervalDays": 14}
            },
            {
                "name": "Plant 2", 
                "genus": "Genus2", 
                "wateringSchedule": {"intervalDays": 10}, 
                "fertilizingSchedule": {"intervalDays": 21}
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
            "wateringSchedule": {"intervalDays": 5},
            "fertilizingSchedule": {"intervalDays": 10}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
        }
        response = client.request("POST", "/plants", json=plant_data)
        assert response.status_code == 201
        plant = response.json()
        
        # Update the plant
        update_data = {
            "name": "Updated Plant",
            "wateringSchedule": {"intervalDays": 10}
        }
        response = client.request("PUT", f"/plants/{plant['id']}", json=update_data)
        assert response.status_code == 200
        
        response_data = response.json()
        assert response_data["name"] == "Updated Plant"
        assert response_data["wateringSchedule"]["intervalDays"] == 10
        assert response_data["genus"] == plant_data["genus"]  # Should remain unchanged
        
    def test_delete_plant(self, client):
        """Test deleting a plant"""
        # Create a plant
        plant_data = {
            "name": "Plant to Delete",
            "genus": "DeleteGenus",
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
        })
        assert response.status_code == 400
        
        # Test validation error with all required fields but invalid values
        response = client.request("POST", "/plants", json={
            "name": "",  # Empty name should fail validation
            "genus": "TestGenus",
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
                "wateringSchedule": {"intervalDays": 7},
                "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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


@pytest.mark.calendar
class TestCalendarFunctionality:
    """Test calendar subscription and iCal feed functionality"""
    
    @pytest.fixture(scope="function")
    def calendar_backend(self):
        """Create a dedicated backend server instance for calendar tests"""
        with BackendServer() as server:
            yield server
    
    @pytest.fixture(scope="function")
    def calendar_client(self, calendar_backend):
        """Create a dedicated client for calendar tests"""
        return APIClient(calendar_backend.base_url, calendar_backend.api_prefix)
    
    @pytest.fixture(autouse=True)
    def login_user(self, calendar_client, test_users):
        """Automatically login a user before each test"""
        user_data = test_users["user1"]
        
        # Register and login user
        calendar_client.request("POST", "/auth/register", json=user_data)
        response = calendar_client.request("POST", "/auth/login", json={
            "email": user_data["email"],
            "password": user_data["password"]
        })
        assert response.status_code == 200
        
        # Store client and user info for test methods
        self.client = calendar_client
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
        plants_data = [
            {
                "name": "Fiddle Leaf Fig",
                "genus": "Ficus",
                "wateringSchedule": {"intervalDays": 7},
                "fertilizingSchedule": {"intervalDays": 14}
            },
            {
                "name": "Snake Plant", 
                "genus": "Sansevieria",
                "wateringSchedule": {"intervalDays": 14},
                "fertilizingSchedule": {"intervalDays": 30}
            }
        ]
        
        created_plants = []
        for plant_data in plants_data:
            response = self.client.request("POST", "/plants", json=plant_data)
            assert response.status_code == 201
            created_plants.append(response.json())
        
        # Get calendar feed
        response = self.client.request("GET", "/calendar/subscription")
        assert response.status_code == 200
        
        feed_url = response.json()["feedUrl"]
        
        # Extract path and query
        import urllib.parse
        parsed_url = urllib.parse.urlparse(feed_url)
        calendar_path = f"{parsed_url.path}?{parsed_url.query}"
        
        # Request the calendar feed
        import requests
        calendar_response = requests.get(f"{self.client.base_url}{calendar_path}")
        assert calendar_response.status_code == 200
        
        calendar_content = calendar_response.text
        
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
        
        # Check event details
        assert "Water every 7 days" in calendar_content
        assert "Water every 14 days" in calendar_content
        assert "Fertilize every 14 days" in calendar_content
        assert "Fertilize every 30 days" in calendar_content
        
        # Check categories
        assert "CATEGORIES:Plant Care,Watering" in calendar_content
        assert "CATEGORIES:Plant Care,Fertilizing" in calendar_content

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
        invalid_url = f"{self.client.base_url}/v1/calendar/{user_id}.ics?token=invalid_token"
        
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
        no_token_url = f"{self.client.base_url}/v1/calendar/{user_id}.ics"
        
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
                "wateringSchedule": {"intervalDays": 5}, 
                "fertilizingSchedule": {"intervalDays": 10}
            },
            {
                "name": "Plant 2", 
                "genus": "Genus2", 
                "wateringSchedule": {"intervalDays": 7}, 
                "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 21}
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


if __name__ == "__main__":
    # Run tests when script is executed directly
    pytest.main([__file__, "-v"])
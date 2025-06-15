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
        env["RUST_LOG"] = "debug,tower_http=info,hyper=info"
        
        try:
            self.process = subprocess.Popen([
                "cargo", "run", "--release", "--bin", "planty-api", "--",
                "--port", str(self.port),
                "--database-url", "sqlite::memory:",
                "--frontend-dir", "/nonexistent"  # Force API-only mode
            ],
                env=env,
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
                "wateringSchedule": {"intervalDays": 7},
                "fertilizingSchedule": {"intervalDays": 14}
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
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14}
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
        
        # Verify the photo was processed and converted to AVIF
        assert photo["contentType"] == "image/avif"
        assert photo["size"] > 0  # Size will be different after AVIF conversion
        assert "width" in photo
        assert "height" in photo


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
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
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
            print(f"Request URL: {self.client.base_url}/v1/plants/{plant_id}/photos")
        assert response.status_code == 201
        photo_data = response.json()
        
        assert "id" in photo_data
        assert photo_data["plantId"] == plant_id
        assert photo_data["originalFilename"] == "test-photo.jpg"
        assert photo_data["contentType"] == "image/avif"  # Images are converted to AVIF
        assert photo_data["size"] > 0  # Size will be different after AVIF conversion
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
        assert updated_plant["thumbnailUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}"
        
        # Verify that fetching the plant individually also returns the thumbnail
        get_response = self.client.request("GET", f"/plants/{plant_id}")
        assert get_response.status_code == 200
        fetched_plant = get_response.json()
        
        assert fetched_plant["thumbnailId"] == photo_id
        assert fetched_plant["thumbnailUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}"
        
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
        assert our_plant["thumbnailUrl"] == f"/api/v1/plants/{plant_id}/photos/{photo_id}"

    def test_async_thumbnail_generation(self):
        """Test asynchronous thumbnail generation during photo upload"""
        # Create a plant first
        plant_data = {
            "name": "Async Test Plant",
            "genus": "Asyncicus",
            "wateringSchedule": {"intervalDays": 5},
            "fertilizingSchedule": {"intervalDays": 12}
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
            f"{self.client.base_url}/v1/plants/{plant_id}/photos",
            files=files,
            cookies=self.client.session.cookies
        )
        
        # Should succeed with proper image
        assert upload_response.status_code == 201
        
        photo = upload_response.json()
        assert "id" in photo
        assert photo["plantId"] == plant_id
        assert photo["originalFilename"] == "async-test.jpg"
        assert photo["contentType"] == "image/avif"  # Should be converted to AVIF
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
        from datetime import datetime, timezone, timedelta
        
        # Create test plants with different schedules and initial care dates
        # Set last watered/fertilized to be clearly in the past to avoid timing issues
        now = datetime.now(timezone.utc)
        plants_data = [
            {
                "name": "Fiddle Leaf Fig",
                "genus": "Ficus",
                "wateringSchedule": {"intervalDays": 7},
                "fertilizingSchedule": {"intervalDays": 14},
                "lastWatered": (now - timedelta(days=6)).isoformat(),  # 6 days ago, so next watering is in 1 day
                "lastFertilized": (now - timedelta(days=13)).isoformat()  # 13 days ago, so next fertilizing is in 1 day
            },
            {
                "name": "Snake Plant", 
                "genus": "Sansevieria",
                "wateringSchedule": {"intervalDays": 14},
                "fertilizingSchedule": {"intervalDays": 30},
                "lastWatered": (now - timedelta(days=13)).isoformat(),  # 13 days ago, so next watering is in 1 day
                "lastFertilized": (now - timedelta(days=29)).isoformat()  # 29 days ago, so next fertilizing is in 1 day
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
            print(f"  Watering schedule: {created_plant.get('wateringSchedule', 'Not found')}")
            print(f"  Fertilizing schedule: {created_plant.get('fertilizingSchedule', 'Not found')}")
            print(f"  Last watered: {created_plant.get('lastWatered', 'Not found')}")
            print(f"  Last fertilized: {created_plant.get('lastFertilized', 'Not found')}")
        
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
        assert "Water every 7 days" in calendar_unwrapped
        assert "Water every 14 days" in calendar_unwrapped
        assert "Fertilize every 14 days" in calendar_unwrapped
        assert "Fertilize every 30 days" in calendar_unwrapped
        
        # Check categories (iCalendar escapes commas with backslashes)
        assert "CATEGORIES:Plant Care\\,Watering" in calendar_content
        assert "CATEGORIES:Plant Care\\,Fertilizing" in calendar_content

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
        from datetime import datetime, timezone, timedelta
        now = datetime.now(timezone.utc)
        
        # Create a plant first
        plant_data = {
            "name": "Test Plant",
            "genus": "Testicus",
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 14},
            "lastWatered": (now - timedelta(days=6)).isoformat(),
            "lastFertilized": (now - timedelta(days=13)).isoformat()
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
        from datetime import datetime, timezone, timedelta
        
        # Create multiple plants with initial care dates
        now = datetime.now(timezone.utc)
        plants_data = [
            {
                "name": "Plant 1", 
                "genus": "Genus1", 
                "wateringSchedule": {"intervalDays": 5}, 
                "fertilizingSchedule": {"intervalDays": 10},
                "lastWatered": (now - timedelta(days=4)).isoformat(),
                "lastFertilized": (now - timedelta(days=9)).isoformat()
            },
            {
                "name": "Plant 2", 
                "genus": "Genus2", 
                "wateringSchedule": {"intervalDays": 7}, 
                "fertilizingSchedule": {"intervalDays": 14},
                "lastWatered": (now - timedelta(days=6)).isoformat(),
                "lastFertilized": (now - timedelta(days=13)).isoformat()
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
        from datetime import datetime, timezone, timedelta
        now = datetime.now(timezone.utc)
        
        # Create plant with unicode characters
        plant_data = {
            "name": "🌿 Monstera Deliciosa",
            "genus": "Mønstéra",
            "wateringSchedule": {"intervalDays": 7},
            "fertilizingSchedule": {"intervalDays": 21},
            "lastWatered": (now - timedelta(days=6)).isoformat(),
            "lastFertilized": (now - timedelta(days=20)).isoformat()
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
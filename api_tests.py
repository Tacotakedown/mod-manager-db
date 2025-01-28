import requests
import os
import base64
import json
from datetime import datetime
from pprint import pformat

class ModAPITester:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url
        self.test_results = []

    def log_result(self, test_name, success, message="", response_data=None):
        result = {
            "test": test_name,
            "success": success,
            "message": message,
            "timestamp": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            "response_data": response_data
        }
        self.test_results.append(result)
        status = "✅" if success else "❌"
        print(f"\n{status} {test_name}: {message}")
        if response_data:
            print("\nResponse Data:")
            try:
                if isinstance(response_data, str):
                    parsed_data = json.loads(response_data)
                    print(json.dumps(parsed_data, indent=2))
                else:
                    print(pformat(response_data, indent=2))
            except json.JSONDecodeError:
                print(response_data)
        print("-" * 50)

    def log_response(self, response):
        """Helper function to extract and format response data"""
        try:
            content_type = response.headers.get('content-type', '')
            if 'application/json' in content_type:
                return response.json()
            elif 'text' in content_type:
                return response.text
            else:
                return f"Binary content ({len(response.content)} bytes)"
        except Exception as e:
            return f"Failed to parse response: {str(e)}"

    def setup_database(self):
        try:
            response = requests.get(f"{self.base_url}/setup")
            response_data = self.log_response(response)
            self.log_result(
                "Database Setup",
                response.status_code == 200,
                f"Status code: {response.status_code}",
                response_data
            )
            return response.status_code == 200
        except requests.RequestException as e:
            self.log_result("Database Setup", False, f"Error: {str(e)}")
            return False

    def create_test_mod_file(self):
        try:
            with open("test_mod.gz", "w") as f:
                f.write("Test mod content")

            with open("test_thumbnail.png", "w") as f:
                f.write("Test thumbnail content")

            self.log_result(
                "Test File Creation",
                True,
                "Test files created successfully",
                {"test_mod.gz": "Created", "test_thumbnail.png": "Created"}
            )
            return True
        except Exception as e:
            self.log_result("Test File Creation", False, f"Error: {str(e)}")
            return False

    def upload_mod(self, mod_id="test-mod-1"):
        try:
            files = {
                'id': (None, mod_id),
                'title': (None, 'Test Mod Title'),
                'version': (None, '1.0.0'),
                'thumbnail': ('thumbnail.png', open('test_thumbnail.png', 'rb')),
                'file': ('mod.gz', open('test_mod.gz', 'rb'))
            }

            request_data = {
                "id": mod_id,
                "title": "Test Mod Title",
                "version": "1.0.0",
                "files": ["thumbnail.png", "mod.gz"]
            }

            response = requests.post(f"{self.base_url}/upload", files=files)
            response_data = self.log_response(response)
            self.log_result(
                "Mod Upload",
                response.status_code == 200,
                f"Status code: {response.status_code}",
                {
                    "request": request_data,
                    "response": response_data
                }
            )
            return response.status_code == 200
        except requests.RequestException as e:
            self.log_result("Mod Upload", False, f"Error: {str(e)}")
            return False
        finally:
            files['thumbnail'][1].close()
            files['file'][1].close()

    def get_metadata(self):
        try:
            response = requests.get(f"{self.base_url}/metadata")
            response_data = self.log_response(response)
            if response.status_code == 200:
                self.log_result(
                    "Get Metadata",
                    True,
                    f"Retrieved metadata. Status code: {response.status_code}",
                    response_data
                )
                return response_data
            else:
                self.log_result(
                    "Get Metadata",
                    False,
                    f"Status code: {response.status_code}",
                    response_data
                )
                return None
        except requests.RequestException as e:
            self.log_result("Get Metadata", False, f"Error: {str(e)}")
            return None

    def download_mod(self, mod_id="test-mod-1"):
        try:
            response = requests.get(f"{self.base_url}/download/{mod_id}")
            if response.status_code == 200:
                with open("downloaded_mod.gz", "wb") as f:
                    f.write(response.content)
                self.log_result(
                    "Mod Download",
                    True,
                    "Mod downloaded successfully",
                    {"size": len(response.content), "mod_id": mod_id}
                )
                return True
            else:
                response_data = self.log_response(response)
                self.log_result(
                    "Mod Download",
                    False,
                    f"Status code: {response.status_code}",
                    response_data
                )
                return False
        except requests.RequestException as e:
            self.log_result("Mod Download", False, f"Error: {str(e)}")
            return False

    def compare_files(self):
        try:
            with open("test_mod.gz", "rb") as f1, open("downloaded_mod.gz", "rb") as f2:
                original_content = f1.read()
                downloaded_content = f2.read()
                matches = original_content == downloaded_content
                comparison_data = {
                    "original_size": len(original_content),
                    "downloaded_size": len(downloaded_content),
                    "match": matches
                }
                self.log_result(
                    "File Comparison",
                    matches,
                    "Files match" if matches else "Files differ",
                    comparison_data
                )
                return matches
        except Exception as e:
            self.log_result("File Comparison", False, f"Error: {str(e)}")
            return False

    def cleanup(self):
        try:
            files_to_remove = ["test_mod.gz", "downloaded_mod.gz", "test_thumbnail.png"]
            removed_files = []
            for file in files_to_remove:
                if os.path.exists(file):
                    os.remove(file)
                    removed_files.append(file)
            self.log_result(
                "Cleanup",
                True,
                "Test files removed successfully",
                {"removed_files": removed_files}
            )
        except Exception as e:
            self.log_result("Cleanup", False, f"Error: {str(e)}")

    def run_all_tests(self):
        print("\n🚀 Starting API tests...\n")
        print("=" * 50)

        self.setup_database()
        self.create_test_mod_file()
        self.upload_mod()
        self.get_metadata()
        self.download_mod()
        self.compare_files()

        self.cleanup()

        print("\n📊 Test Summary:")
        print("=" * 50)
        total_tests = len(self.test_results)
        passed_tests = sum(1 for result in self.test_results if result["success"])
        print(f"Total Tests: {total_tests}")
        print(f"Passed: {passed_tests}")
        print(f"Failed: {total_tests - passed_tests}")

        with open('test_results.json', 'w') as f:
            json.dump(self.test_results, f, indent=2)
        print("\nDetailed test results saved to 'test_results.json'")

if __name__ == "__main__":
    tester = ModAPITester()
    tester.run_all_tests()
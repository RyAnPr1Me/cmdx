@echo off
REM Example Windows batch script
echo Starting Windows script execution...
echo.

REM List directory contents
echo Listing current directory:
dir /b

REM Create a test file
echo Hello from Windows > test_output.txt

REM Display file contents  
echo.
echo File contents:
type test_output.txt

REM Cleanup
del test_output.txt

echo.
echo Script completed successfully!

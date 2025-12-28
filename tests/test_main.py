"""Tests for the main module."""

from unittest.mock import patch

from modal_dictation_exploration.main import main


class TestMain:
    """Tests for the main() function."""

    def test_logs_startup_message(self, pystray_mock, caplog):
        """Should log startup message."""
        import logging
        with caplog.at_level(logging.INFO):
            main()

        assert any("started" in record.message.lower() for record in caplog.records)

    @patch('modal_dictation_exploration.main.setup_tray')
    def test_calls_setup_tray(self, mock_setup_tray):
        """Should call setup_tray to create the icon."""
        main()

        mock_setup_tray.assert_called_once()

    @patch('modal_dictation_exploration.main.setup_tray')
    def test_calls_icon_run(self, mock_setup_tray):
        """Should call run() on the icon returned by setup_tray."""
        main()

        mock_setup_tray.return_value.run.assert_called_once()

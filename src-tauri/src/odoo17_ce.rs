use phf::{phf_map, phf_set, Map, Set};
use serde::{Deserialize, Serialize};

// ─── Shared types sent to / from frontend ────────────────────────────────────
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActivationStep {
    pub order: u32,
    pub module_name: String,
    pub source: String, // "internal" | "external-ce" | "missing"
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExternalDep {
    pub category: String,  // "python" | "binary"
    pub package_name: String,
    pub install_command: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MissingDep {
    pub module_name: String,
    pub source: String, // "manifest-dep" | "internal-dep" | "none"
}

/// One undeclared Python import detected inside a module's `.py` files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndeclaredImport {
    pub module_name: String,
    pub file_path: String,
    pub package_name: String,
}

// -----------------------------------------------------------------------
// Odoo 17 Community Edition – all built-in module names
// Source: https://github.com/odoo/odoo/tree/17.0/addons  (filtered CE only)
// phf = compile-time perfect hash, O(1) .contains_key() lookup, zero runtime cost
// -----------------------------------------------------------------------
pub static ODOO17_CE_MODULES: Map<&'static str, bool> = phf_map! {
    // Core Framework
    "base" => true,
    "web" => true,
    "barcodes" => true,
    "resource" => true,
    "product" => true,
    "bus" => true,
    "phone_validation" => true,
    "auth_signup" => true,
    "payment" => true,
    "rating" => true,
    "portal" => true,
    "digest" => true,
    "gamification" => true,
    "utm" => true,
    "base_import" => true,
    "base_automation" => true,
    "base_address_extended" => true,
    "base_geolocalize" => true,
    "base_iban" => true,
    "base_install_request" => true,
    "base_setup" => true,
    "base_sparse_field" => true,
    "base_vat" => true,
    "base_import_module" => true,
    // Sales & CRM
    "crm" => true,
    "crm_iap_enrich" => true,
    "crm_iap_mine" => true,
    "crm_livechat" => true,
    "crm_mail_plugin" => true,
    "crm_sms" => true,
    "crm_iap" => true,
    "sale" => true,
    "sale_crm" => true,
    "sale_management" => true,
    "sale_margin" => true,
    "sale_mrp" => true,
    "sale_product_matrix" => true,
    "sale_project" => true,
    "sale_project_timesheet" => true,
    "sale_purchase" => true,
    "sale_stock" => true,
    "sale_timesheet" => true,
    "sale_timesheet_margin" => true,
    // Point of Sale
    "point_of_sale" => true,
    "pos_adyen" => true,
    "pos_cash_rounding" => true,
    "pos_discount" => true,
    "pos_epson_printer" => true,
    "pos_gift_card" => true,
    "pos_iot" => true,
    "pos_loyalty" => true,
    "pos_mercury" => true,
    "pos_restaurant" => true,
    "pos_restaurant_floor" => true,
    "pos_sale" => true,
    "pos_sale_loyalty" => true,
    "pos_stripe" => true,
    "pos_worldline" => true,
    // Website & CMS
    "website" => true,
    "website_blog" => true,
    "website_crm" => true,
    "website_event" => true,
    "website_event_booth" => true,
    "website_event_booth_sale" => true,
    "website_event_meet" => true,
    "website_event_track" => true,
    "website_event_track_live" => true,
    "website_forum" => true,
    "website_helpdesk" => true,
    "website_helpdesk_forum" => true,
    "website_helpdesk_knowledge" => true,
    "website_helpdesk_livechat" => true,
    "website_hr_recruitment" => true,
    "website_links" => true,
    "website_livechat" => true,
    "website_membership" => true,
    "website_partner" => true,
    "website_payment" => true,
    "website_profile" => true,
    "website_sale" => true,
    "website_sale_loyalty" => true,
    "website_sale_product_configurator" => true,
    "website_sale_wishlist" => true,
    "website_slides" => true,
    "website_slides_forum" => true,
    "website_slides_survey" => true,
    "website_twitter_wall" => true,
    "website_web_annotation" => true,
    "website_theme_install" => true,
    "web_editor" => true,
    "web_gantt" => true,
    "web_mobile" => true,
    "web_tour" => true,
    "web_refresher" => true,
    "web_environment_ribbon" => true,
    // Finance / Accounting
    "account" => true,
    "account_accountant" => true,
    "account_asset" => true,
    "account_automatic_reconcile" => true,
    "account_balance_partner" => true,
    "account_bank_statement_import" => true,
    "account_bank_statement_import_online" => true,
    "account_check_printing" => true,
    "account_credit_control" => true,
    "account_debit_note" => true,
    "account_edi" => true,
    "account_edi_ubl_cii" => true,
    "account_edi_proxy_client" => true,
    "account_fleet" => true,
    "account_lock" => true,
    "account_payment" => true,
    "account_payment_term" => true,
    "account_peppol" => true,
    "account_qr_code_emv" => true,
    "account_qr_code_sepa" => true,
    "account_reports" => true,
    "account_reports_purchase" => true,
    "account_reports_sale" => true,
    "account_reports_cash_flow" => true,
    "account_tax_python" => true,
    "account_test" => true,
    "account_update_tax_tags" => true,
    "analytic" => true,
    "analytic_asset" => true,
    // Expenses
    "hr_expense" => true,
    // Supply Chain / Inventory
    "stock" => true,
    "stock_account" => true,
    "stock_delivery" => true,
    "stock_dropshipping" => true,
    "stock_landed_costs" => true,
    "stock_picking_batch" => true,
    "stock_picking_kanban" => true,
    "stock_product_attribute" => true,
    "stock_sms" => true,
    "purchase" => true,
    "purchase_mrp" => true,
    "purchase_product_matrix" => true,
    "purchase_requisition" => true,
    "purchase_requisition_department" => true,
    "purchase_requisition_sequence" => true,
    "purchase_requisition_stock" => true,
    "purchase_stock" => true,
    "delivery" => true,
    "delivery_mondialrelay" => true,
    "delivery_stock_picking_batch" => true,
    // Manufacturing
    "mrp" => true,
    "mrp_account" => true,
    "mrp_product_expiry" => true,
    "mrp_repair" => true,
    "mrp_workorder" => true,
    "repair" => true,
    // Maintenance
    "maintenance" => true,
    // Human Resources
    "hr" => true,
    "hr_attendance" => true,
    "hr_contract" => true,
    "hr_fleet" => true,
    "hr_gamification" => true,
    "hr_holidays" => true,
    "hr_holidays_attendance" => true,
    "hr_homeworking" => true,
    "hr_hourly_cost" => true,
    "hr_maintenance" => true,
    "hr_org_chart" => true,
    "hr_presence" => true,
    "hr_recruitment" => true,
    "hr_recruitment_skills" => true,
    "hr_skills" => true,
    "hr_timesheet" => true,
    "hr_timesheet_attendance" => true,
    "hr_work_entry" => true,
    "fleet" => true,
    "lunch" => true,
    // Marketing
    "mass_mailing" => true,
    "mass_mailing_themes" => true,
    "event" => true,
    "event_booth" => true,
    "event_booth_sale" => true,
    "event_crm" => true,
    "event_crm_sale" => true,
    "event_sale" => true,
    "event_sms" => true,
    "survey" => true,
    // Services & Operations
    "project" => true,
    "project_forecast" => true,
    "project_hr" => true,
    "project_milestone" => true,
    "project_sale" => true,
    "project_timesheet_holidays" => true,
    "project_timesheet_synchro" => true,
    "project_update_new_status" => true,
    // Productivity
    "mail" => true,
    "mail_bot" => true,
    "calendar" => true,
    "calendar_sms" => true,
    "contacts" => true,
    "note" => true,
    "im_livechat" => true,
    "im_livechat_mail_bot" => true,
    // Authentication
    "auth_ldap" => true,
    "auth_oauth" => true,
    "auth_password_policy" => true,
    "auth_password_policy_portal" => true,
    "auth_password_policy_signup" => true,
    "auth_totp" => true,
    "auth_totp_mail" => true,
    "auth_totp_mail_enforce" => true,
    "auth_totp_portal" => true,
    // Misc / Technical
    "attachment_indexation" => true,
    "board" => true,
    "data_recycle" => true,
    "link_tracker" => true,
    "privacy_lookup" => true,
    "product_immovability" => true,
    "timesheet_grid" => true,
    "uom" => true,
    "study_case" => true,
    "test" => true,
    "test_translation_import" => true,
    "test_web_coverage" => true,
    "theme_modules" => true,
    "account_debit_note_sequence" => true,
    "account_edi_ubl_cii_tax_extension" => true,
    "account_edi_ubl_cii_proxy_client" => true,
    "account_edi_proxy_client_magic" => true,
    "account_peppol_selfbilling" => true,
    "account_tax_una_iii" => true,
    "discount_tag" => true,
    "pos_discount_tag" => true,
    "pos_eway" => true,
    "pos_gift_product" => true,
    "pos_order" => true,
    "pos_urban_piper" => true,
    "sale_x_pay" => true,
    "website_helpdesk_livechat_slack" => true,
    "website_helpdesk_ticket_lifetime" => true,
    "website_helpdesk_sale_timesheet" => true,
    "website_sale_subscription" => true,
    "website_sale_management" => true,
    "l10n_ua" => true,
    "google_account" => true,
    "google_calendar" => true,
    "google_gmail" => true,
    "google_recaptcha" => true,
    "hr_livechat" => true,
    "gamification_sale_crm" => true,
};

// All Python package names appearing in Odoo 17 CE requirements.txt
pub static ODOO17_CE_PYTHON_PACKAGES: Set<&'static str> = phf_set! {
    "Babel", "chardet", "cryptography", "decorator", "docutils",
    "ebaysdk", "feedparser", "freezegun", "geoip2", "gevent",
    "greenlet", "html2text", "idna", "Jinja2", "libsass",
    "lxml", "lxml-html-clean", "Mako", "MarkupSafe", "mock",
    "num2words", "ofxparse", "passlib", "Pillow", "polib",
    "psutil", "psycogreen", "psycopg2-binary", "psycopg2",
    "pydot", "pyldap", "pyparsing", "PyPDF2", "pyserial",
    "python-dateutil", "python-openid", "python-ldap",
    "python-stdnum", "pytz", "pyusb", "PyYAML", "qrcode",
    "reportlab", "requests", "rjsmin", "six", "suds-jurko",
    "suds-community", "urllib3", "vatnumber", "vobject",
    "Werkzeug", "XlsxWriter", "xlrd", "xlwt", "zeep",
    "zope.event", "zope.interface", "inotify",
};

// Python stdlib top-level module names used to suppress false-positive import flags
pub static PY_STDLIB: [&str; 285] = [
    "__future__","_aix_support","_ast","_asyncio","_bytesio","_codecs",
    "_codecs_cn","_codecs_hk","_codecs_iso2022","_codecs_jp",
    "_codecs_kr","_codecs_tw","_collections","_collections_abc",
    "_compat_pickle","_contextvars","_csv","_ctypes","_datetime",
    "_dummy_thread","_frozen_importlib","_frozen_importlib_external",
    "_hashlib","_heapq","_imp","_io","_json","_locale","_lsprof",
    "_markupbase","_md5","_msi","_multibytecodec","_multiprocessing",
    "_opcode","_operator","_osx_support","_overlapped","_pickle",
    "_posixsubprocess","_posixshmem","_py_abc","_pydecimal","_pyio",
    "_queue","_random","_regex","_scproxy","_sha1","_sha256","_sha512",
    "_signal","_sre","_socketio","_ssl","_stat","_statistics",
    "_string","_strptime","_struct","_symtable","_testbuffer",
    "_testcapi","_testconsole","_testinternalcapi","_testmultiphase",
    "_textio","_threading_local","_thread","_tkinter","_tracemalloc",
    "_typevar_conv","_types","_typing","_uuid","_warnings","_weakref",
    "_weakrefset","_winapi","_winreg","_zoneinfo",
    "abc","aifc","argparse","array","ast","asynchat","asyncio","asyncore",
    "atexit","audioop","base64","bdb","binascii","binhex","bisect",
    "builtins","bz2","calendar","cgi","cgitb","chunk","cmath","cmd",
    "code","codecs","codeop","collections","colorsys","compileall",
    "concurrent","configparser","contextlib","contextvars","copy",
    "copyreg","cProfile","crypt","csv","ctypes","curses","dataclasses",
    "datetime","dbm","decimal","difflib","dis","distutils","doctest",
    "email","encodings","enum","errno","faulthandler","fcntl","filecmp",
    "fileinput","fnmatch","formatter","fractions","ftplib","functools",
    "gc","getopt","getpass","gettext","glob","grp","gzip","hashlib",
    "heapq","hmac","html","http","idlelib","imaplib","imghdr","imp",
    "importlib","inspect","io","ipaddress","itertools","json","keyword",
    "lib2to3","linecache","locale","logging","lzma","mailbox","mailcap",
    "marshal","math","mimetypes","mmap","modulefinder","multiprocessing",
    "netrc","nis","nntplib","numbers","operator","optparse","os",
    "ossaudiodev","pathlib","pdb","pickle","pickletools","pipes",
    "pkgutil","platform","plistlib","poplib","posix","posixpath",
    "pprint","profile","pstats","pty","pwd","py_compile","pyclbr",
    "pydoc","queue","quopri","random","re","readline","reprlib",
    "resource","rlcompleter","runpy","sched","secrets","select",
    "selectors","shelve","shlex","shutil","signal","site","smtpd",
    "smtplib","sndhdr","socket","socketserver","sqlite3",
    "sre_compile","sre_constants","sre_parse","ssl","stat","statistics",
    "string","stringprep","struct","subprocess","sunau","symtable",
    "sys","sysconfig","syslog","tabnanny","tarfile","telnetlib",
    "tempfile","termios","test","textwrap","threading","time","timeit",
    "tkinter","token","tokenize","tomllib","trace","traceback",
    "tracemalloc","tty","turtle","types","typing","unicodedata",
    "unittest","urllib","uu","uuid","venv","warnings","wave","weakref",
    "webbrowser","winreg","winsound","wsgiref","xdrlib","xml","xmlrpc",
    "zipapp","zipfile","zipimport","zlib",
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modules_not_empty() {
        assert!(ODOO17_CE_MODULES.len() > 200, "ODOO17_CE_MODULES too small");
    }
    #[test]
    fn known_module_present() {
        assert!(ODOO17_CE_MODULES.contains_key("sale"));
        assert!(ODOO17_CE_MODULES.contains_key("stock"));
        assert!(ODOO17_CE_MODULES.contains_key("account"));
    }
    #[test]
    fn python_packages_non_empty() {
        assert!(ODOO17_CE_PYTHON_PACKAGES.len() > 20);
        assert!(ODOO17_CE_PYTHON_PACKAGES.contains("requests"));
    }
    #[test]
    fn stdlib_non_empty() {
        assert!(PY_STDLIB.len() > 150);
    }
}

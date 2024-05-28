



"""
1. Run the Odoo client to extract the full tables
2. Run the Sankey Python Dash to generate the Sankey diagram

"""


from odoo_client import OdooClient

import os
from dotenv import load_dotenv

load_dotenv()


def fetch_odoo_data():

    client = OdooClient()

    queries = []

    # Queries are in the format (model, domain, fields)
    # These queries are for the Sankey diagram and are essentially SELECT statements returning all rows and columns specified

    _fields = ["access_url", "access_warning", "account_number", "activity_calendar_event_id", "activity_date_deadline", "activity_exception_decoration", "activity_exception_icon", "activity_ids", "activity_state", "activity_summary", "activity_type_icon", "activity_type_id", "activity_user_id", "always_tax_exigible", "amount", "amount_currency", "amount_paid", "amount_residual", "amount_residual_signed", "amount_tax", "amount_tax_signed", "amount_total", "amount_total_in_currency_signed", "amount_total_signed", "amount_untaxed", "amount_untaxed_signed", "asset_asset_type", "asset_depreciated_value", "asset_id", "asset_id_display_name", "asset_ids", "asset_ids_display_name", "asset_manually_modified", "asset_remaining_value", "asset_value_change", "attachment_ids", "authorized_transaction_ids", "auto_post", "bank_partner_id", "campaign_id", "commercial_partner_id", "company_currency_id", "company_id", "country_code", "create_date", "create_uid", "currency_id", "date", "display_inactive_currency_warning", "display_name", "display_qr_code", "draft_asset_ids", "duplicated_vendor_ref", "edi_blocking_level", "edi_document_ids", "edi_error_count", "edi_error_message", "edi_show_abandon_cancel_button", "edi_show_cancel_button", "edi_state", "edi_web_services_to_process", "event_id", "extract_can_show_resend_button", "extract_can_show_send_button", "extract_error_message", "extract_remote_id", "extract_state", "extract_status_code", "extract_word_ids", "fiscal_position_id", "foreign_currency_id", "has_message", "has_reconciled_entries", "highest_name", "id", "inalterable_hash", "invoice_cash_rounding_id", "invoice_date", "invoice_date_due", "invoice_filter_type_domain", "invoice_has_matching_suspense_amount", "invoice_has_outstanding", "invoice_incoterm_id", "invoice_line_ids", "invoice_origin", "invoice_outstanding_credits_debits_widget", "invoice_partner_display_name", "invoice_payment_term_id", "invoice_payments_widget", "invoice_source_email", "invoice_user_id", "invoice_vendor_bill_id", "is_move_sent", "is_reconciled", "is_taxcloud", "is_taxcloud_configured", "journal_id", "line_ids", "medium_id", "message_attachment_count", "message_follower_ids", "message_has_error", "message_has_error_counter", "message_has_sms_error", "message_ids", "message_is_follower", "message_main_attachment_id", "message_needaction", "message_needaction_counter", "message_partner_ids", "message_unread", "message_unread_counter", "move_id", "move_type", "my_activity_date_deadline", "name", "narration", "number_asset_ids", "online_account_id", "online_link_id", "online_partner_information", "online_transaction_identifier", "partner_bank_id", "partner_id", "partner_name", "partner_shipping_id", "payment_id", "payment_ids", "payment_ref", "payment_reference", "payment_state", "payment_state_before_switch", "posted_before", "purchase_id", "purchase_vendor_bill_id", "qr_code_method", "ref", "restrict_mode_hash_table", "reversal_move_id", "reversed_entry_id", "secure_sequence_number", "sequence", "sequence_number", "sequence_prefix", "show_name_warning", "show_reset_to_draft_button", "source_id", "state", "statement_id", "statement_line_id", "stock_move_id", "stock_valuation_layer_ids", "string_to_hash", "suitable_journal_ids", "tax_cash_basis_created_move_ids", "tax_cash_basis_origin_move_id", "tax_cash_basis_rec_id", "tax_closing_end_date", "tax_country_code", "tax_country_id", "tax_lock_date_message", "tax_report_control_error", "tax_totals_json", "team_id", "to_check", "transaction_ids", "transaction_type", "transfer_model_id", "type_name", "unique_import_id", "user_id", "website_message_ids", "write_date", "write_uid"]

    query = ("account.bank.statement.line", [], _fields)
    queries.append(query)

    query = ("event.event", [], ["id", "event_name", "end_datetime", "start_datetime", "invoice_ids", "purchase_order_ids", "rental_order_ids"])
    queries.append(query)

    query = ("account.move", [], ['id', 'name', 'date', 'state', 'event_id', 'journal_id', 'amount_total', 'partner_id', 'purchase_id', 'payment_id', 'payment_state', 'move_type', 'type_name', 'invoice_line_ids', "invoice_origin"])
    queries.append(query)

    query = ("account.move.line", [], ['id', 'move_id', 'display_name', 'date', 'account_id', 'journal_id', 'partner_id', 'sale_line_ids', 'purchase_order_id', 'purchase_line_id', 'subscription_id', 'payment_id', 'product_id', 'product_uom_category_id', 'quantity', 'credit', 'debit', 'account_internal_type', 'account_internal_group', 'display_type'])
    queries.append(query)

    query = ("sale.order", [], ["id", "state", "name", "partner_id", "event_id", "state"])
    queries.append(query)

    query = ("sale.order.line", [], ["id", "order_id", "product_id", "price_total", "invoice_status", "product_uom_qty", "product_uom_category_id"])
    queries.append(query)

    query = ("product.product", [], ["id", "name", "categ_id"])
    queries.append(query)

    query = ("purchase.order", [], ["id", "name", "partner_id"])
    queries.append(query)

    query = ("purchase.order.line", [], ["id", "order_id", "product_id", "price_total", "state", "product_uom_category_id"])
    queries.append(query)

    query = ("crm.lead", [], ["id", "name", "probability", "stage_id", "lost_reason", "days_to_convert", "campaign_id", "event_id"])
    queries.append(query)

    # The sign.item model contains the field ids that are included in the sign.request.item.value model
    query = ("sign.item", [], ["id", "display_name", "name", "required", "responsible_id", "template_id", "type_id"])
    queries.append(query)

    # The Sign Request
    query = ("sign.request", [], ["id", "display_name", "reference", "state", "completion_date", "request_item_ids", "request_item_infos"])
    queries.append(query)

    # The sign.request.item annotates sign.request.item.value with the name of the field which is being signed
    query = ("sign.request.item", [], ["id", "display_name", "reference", "sign_request_id", "signing_date", "signer_email", "partner_id", "sign_item_value_ids"])
    queries.append(query)

    query = ("sign.request.item.value", [], ["id", "display_name", "sign_item_id", "sign_request_id", "sign_request_item_id", "value"])
    queries.append(query)

    query = ("res.partner", [], ["id", "name", "email", "phone", "commercial_company_name", "box_1099_id"])
    queries.append(query)

    for query in queries:

        df = client.search_read(query)

        df.to_csv(f'/workspace/submodules/sjp/sankey/src/sankey-python-dash/data/{query[0]}.csv')



if __name__ == "__main__":
    REFRESH = True
    if REFRESH:
        print("Fetching Odoo Data")
        fetch_odoo_data()
    # print("Running Dash App")
    # app.run_server(debug=True)